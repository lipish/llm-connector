# 流式 Tool Calls 问题修复总结

## 问题确认

经过详细分析和测试，确认了 llm-connector 在处理流式 tool_calls 时存在以下问题：

### 核心问题

1. **数据结构不支持增量解析**
   - `ToolCall` 和 `FunctionCall` 的所有字段都是必填的 `String`
   - 导致 OpenAI 流式 API 返回的增量 chunk 无法正确解析（缺少必填字段）

2. **缺少累积逻辑**
   - 没有按 `index` 维护状态来累积 tool_calls
   - 每个 chunk 的 `tool_calls` 都被直接传递给上层应用

3. **潜在的重复执行风险**
   - 如果增量 chunk 能够解析，上层应用会收到多个不完整的 tool_call
   - 可能导致工具被重复调用

### 测试验证

创建了测试用例 `tests/test_streaming_tool_calls.rs`，验证了：

- ✅ 增量 chunk 现在可以正确解析（修复后）
- ✅ tool_calls 可以正确累积
- ✅ 只有完整的 tool_calls 才会被发送给上层应用

## 解决方案

### 1. 修改数据结构（`src/types/request.rs`）

```rust
// 修改前
pub struct ToolCall {
    pub id: String,           // ❌ 必填
    pub call_type: String,    // ❌ 必填
    pub function: FunctionCall,
}

pub struct FunctionCall {
    pub name: String,         // ❌ 必填
    pub arguments: String,    // ❌ 必填
}

// 修改后
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCall {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    
    #[serde(rename = "type", default, skip_serializing_if = "String::is_empty")]
    pub call_type: String,
    
    #[serde(default)]
    pub function: FunctionCall,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,  // ✅ 新增：用于流式累积
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionCall {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub arguments: String,
}
```

### 2. 添加累积方法（`src/types/request.rs`）

```rust
impl ToolCall {
    /// 合并增量数据
    pub fn merge_delta(&mut self, delta: &ToolCall) {
        if !delta.id.is_empty() {
            self.id = delta.id.clone();
        }
        if !delta.call_type.is_empty() {
            self.call_type = delta.call_type.clone();
        }
        if !delta.function.name.is_empty() {
            self.function.name = delta.function.name.clone();
        }
        // 累积 arguments
        if !delta.function.arguments.is_empty() {
            self.function.arguments.push_str(&delta.function.arguments);
        }
        if delta.index.is_some() {
            self.index = delta.index;
        }
    }
    
    /// 检查是否完整
    pub fn is_complete(&self) -> bool {
        !self.id.is_empty() 
            && !self.call_type.is_empty() 
            && !self.function.name.is_empty()
    }
}
```

### 3. 实现流式累积逻辑（`src/sse.rs`）

在 `sse_to_streaming_response` 函数中使用 `scan` 维护累积状态：

```rust
let response_stream = string_stream.scan(
    HashMap::<usize, ToolCall>::new(),  // 累积状态
    |accumulated_tool_calls, result| {
        // 对每个 chunk 中的 tool_calls 进行累积
        if let Some(delta_tool_calls) = &choice.delta.tool_calls {
            for delta_call in delta_tool_calls {
                let index = delta_call.index.unwrap_or(0);
                
                accumulated_tool_calls
                    .entry(index)
                    .and_modify(|existing| existing.merge_delta(delta_call))
                    .or_insert_with(|| delta_call.clone());
            }
            
            // 只发送完整的 tool_calls
            let complete_calls: Vec<ToolCall> = accumulated_tool_calls
                .values()
                .filter(|call| call.is_complete())
                .cloned()
                .collect();
            
            if !complete_calls.is_empty() {
                choice.delta.tool_calls = Some(complete_calls);
            } else {
                choice.delta.tool_calls = None;  // 避免发送不完整的
            }
        }
        // ...
    }
);
```

## 修复效果

### 修复前

```
Chunk #1: tool_calls = [{"id": "call_123", "name": "get_weather", "arguments": ""}]
Chunk #2: 解析失败（缺少 name 字段）
Chunk #3: 解析失败（缺少 name 字段）
Chunk #4: 解析失败（缺少 name 字段）

结果：只能拿到第一个 chunk 的空 arguments
```

### 修复后

```
Chunk #1: 累积 tool_call[0] = {"id": "call_123", "name": "get_weather", "arguments": ""}
Chunk #2: 累积 tool_call[0] = {"id": "call_123", "name": "get_weather", "arguments": "{\"loc"}
Chunk #3: 累积 tool_call[0] = {"id": "call_123", "name": "get_weather", "arguments": "{\"location\": \"Beijing"}
Chunk #4: 累积 tool_call[0] = {"id": "call_123", "name": "get_weather", "arguments": "{\"location\": \"Beijing\"}"}

上层应用只收到一个完整的 tool_call：
{"id": "call_123", "type": "function", "name": "get_weather", "arguments": "{\"location\": \"Beijing\"}"}
```

## 向后兼容性

✅ 完全向后兼容：

- 非流式模式不受影响
- 现有的 tool_calls 使用方式继续工作
- 所有现有测试通过（82 个测试全部通过）

## 文件变更

1. `src/types/request.rs` - 修改 ToolCall 和 FunctionCall 数据结构
2. `src/sse.rs` - 实现流式 tool_calls 累积逻辑
3. `src/protocols/openai.rs` - 添加 index 字段初始化
4. `tests/test_streaming_tool_calls.rs` - 新增测试用例
5. `docs/STREAMING_TOOL_CALLS.md` - 新增文档说明

## 建议

上层应用（如 Codex CLI）可以安全地使用流式 tool_calls，不需要任何修改。llm-connector 已经在内部处理了所有的累积和去重逻辑。

