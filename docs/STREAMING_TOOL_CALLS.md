# 流式 Tool Calls 支持

## 问题背景

在 OpenAI 的流式 API 中，`tool_calls` 是通过多个 chunk 增量发送的：

```json
// Chunk 1 - 开始 tool_call
{"choices": [{"delta": {"tool_calls": [{"index": 0, "id": "call_123", "type": "function", "function": {"name": "get_weather", "arguments": ""}}]}}]}

// Chunk 2 - 参数增量
{"choices": [{"delta": {"tool_calls": [{"index": 0, "function": {"arguments": "{\"loc"}}]}}]}

// Chunk 3 - 参数增量
{"choices": [{"delta": {"tool_calls": [{"index": 0, "function": {"arguments": "ation\": \"Beijing"}}]}}]}

// Chunk 4 - 参数增量
{"choices": [{"delta": {"tool_calls": [{"index": 0, "function": {"arguments": "\"}"}}]}}]}

// Chunk 5 - 结束
{"choices": [{"delta": {}, "finish_reason": "tool_calls"}]}
```

### 原始问题

在修复前，llm-connector 存在以下问题：

1. **数据结构不支持增量**：`ToolCall` 和 `FunctionCall` 的所有字段都是必填的 `String`，导致增量 chunk 解析失败
2. **缺少累积逻辑**：没有按 `index` 累积 tool_calls，每个 chunk 都被当作独立的 tool_call
3. **可能导致重复执行**：上层应用收到多个不完整的 tool_call，可能导致重复执行

## 解决方案

### 1. 修改数据结构支持增量

将 `ToolCall` 和 `FunctionCall` 的字段改为可选，并添加 `Default` trait：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCall {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    
    #[serde(rename = "type", default, skip_serializing_if = "String::is_empty")]
    pub call_type: String,
    
    #[serde(default)]
    pub function: FunctionCall,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,  // 用于流式累积
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionCall {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub arguments: String,
}
```

### 2. 实现累积逻辑

添加 `merge_delta` 方法用于合并增量数据：

```rust
impl ToolCall {
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
    
    pub fn is_complete(&self) -> bool {
        !self.id.is_empty() 
            && !self.call_type.is_empty() 
            && !self.function.name.is_empty()
    }
}
```

### 3. 在流式解析器中集成累积逻辑

在 `src/sse.rs` 的 `sse_to_streaming_response` 函数中：

```rust
pub fn sse_to_streaming_response(response: reqwest::Response) -> ChatStream {
    let string_stream = sse_events(response);
    
    // 使用 scan 维护累积状态
    let response_stream = string_stream.scan(
        HashMap::<usize, ToolCall>::new(),
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
    
    Box::pin(response_stream)
}
```

## 使用示例

修复后，上层应用可以安全地使用流式 tool_calls：

```rust
let mut stream = client.chat_stream(&request).await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    
    if let Some(choice) = chunk.choices.first() {
        // 只会收到完整的 tool_calls，不会重复
        if let Some(tool_calls) = &choice.delta.tool_calls {
            for call in tool_calls {
                println!("Tool Call: {}", call.function.name);
                println!("Arguments: {}", call.function.arguments);
                // 安全执行，不会重复
                execute_tool(&call);
            }
        }
    }
}
```

## 测试验证

参见 `tests/test_streaming_tool_calls.rs`：

- `test_streaming_tool_calls_accumulation`: 验证累积逻辑正确
- `test_streaming_tool_calls_parsing`: 展示原始问题（增量 chunk 可以解析）

## 向后兼容性

这个修复完全向后兼容：

- 非流式模式不受影响
- 现有的 tool_calls 使用方式继续工作
- 只是修复了流式模式下的累积逻辑

