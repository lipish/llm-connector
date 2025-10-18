# 测试智谱 GLM 流式响应问题

## 问题描述

需要验证智谱 GLM 系列模型是否存在以下问题：

> ❌ **有问题的场景**：只有在请求包含 `Role::Tool` 消息（即工具执行结果）时，GLM 系列模型的流式 API 才会返回空内容。

## 测试场景

### 场景 1: 第一轮请求（无 `Role::Tool` 消息）
- 用户提问触发工具调用
- 应该正常返回流式响应
- 预期：多个流式块，包含工具调用信息

### 场景 2: 第二轮请求（包含 `Role::Tool` 消息）
- 包含工具执行结果
- 检查流式响应是否返回空内容
- 预期：如果有问题，会返回空内容或只有 1 个块

## 运行测试

### 前置条件

1. 设置智谱 API Key：
```bash
export ZHIPU_API_KEY="your-zhipu-api-key"
```

2. 确保已启用 streaming 功能

### 执行测试

```bash
cargo run --example test_zhipu_tool_streaming_issue --features streaming
```

## 测试模型

测试将依次验证以下模型：
- `glm-4-flash`
- `glm-4`
- `glm-4.5`

## 预期输出

### 正常情况（无问题）

```
📝 测试模型: glm-4.5

✅ 场景 1: 第一轮请求（无 Role::Tool 消息）
   finish_reason: tool_calls
   收到流式块: 91 个
   内容长度: 0 字符
   有工具调用: true

⚠️  场景 2: 第二轮请求（包含 Role::Tool 消息）
   finish_reason: stop
   收到流式块: 85 个
   内容长度: 245 字符
   内容预览: 根据查询结果，上海目前的天气情况如下：...

📊 对比分析:
   场景 1（无 Tool）: 91 块, 0 字符
   场景 2（有 Tool）: 85 块, 245 字符

   ✅ 正常: 包含 Role::Tool 时流式响应正常
```

### 异常情况（有问题）

```
📝 测试模型: glm-4.5

✅ 场景 1: 第一轮请求（无 Role::Tool 消息）
   finish_reason: tool_calls
   收到流式块: 91 个
   内容长度: 0 字符
   有工具调用: true

⚠️  场景 2: 第二轮请求（包含 Role::Tool 消息）
   finish_reason: stop
   收到流式块: 1 个
   内容长度: 0 字符
   ⚠️  内容为空！

📊 对比分析:
   场景 1（无 Tool）: 91 块, 0 字符
   场景 2（有 Tool）: 1 块, 0 字符

   ❌ 问题确认: 包含 Role::Tool 时流式返回空内容！
```

或者：

```
📊 对比分析:
   场景 1（无 Tool）: 91 块, 0 字符
   场景 2（有 Tool）: 1 块, 245 字符

   ⚠️  可能的问题: 流式块数量显著减少（可能被强制切换为非流式）
```

## 问题分析

### 如果确认有问题

可能的原因：
1. **智谱 API 本身的限制**：API 在处理包含 `Role::Tool` 的请求时不支持流式响应
2. **协议实现问题**：请求构建时某些字段导致 API 无法正确处理流式
3. **响应解析问题**：流式响应的解析逻辑在处理工具调用结果时有 bug

### 解决方案

如果确认是智谱 API 的限制，可能需要：

1. **恢复之前的修复逻辑**（在 `src/core/traits.rs` 中）：
```rust
#[cfg(feature = "streaming")]
async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
    use crate::types::Role;
    use futures_util::stream;
    
    // 智谱 API 在包含 Role::Tool 时不支持流式响应
    let has_tool_messages = request.messages.iter().any(|m| m.role == Role::Tool);
    
    if has_tool_messages && self.protocol.name() == "zhipu" {
        // 降级为非流式请求
        let response = self.chat(request).await?;
        let single_response = stream::once(async move { Ok(response.into()) });
        return Ok(Box::pin(single_response));
    }
    
    // 正常流式处理
    // ...
}
```

2. **添加配置选项**：让用户选择是否启用这个修复

3. **联系智谱官方**：确认是否是 API 的已知限制

## 相关文件

- 测试代码：`examples/test_zhipu_tool_streaming_issue.rs`
- 核心实现：`src/core/traits.rs`
- 智谱协议：`src/providers/zhipu.rs`

## 注意事项

1. 测试需要真实的 API Key 和网络连接
2. 测试会消耗 API 配额（每个模型 2 次请求）
3. 如果第一轮未触发工具调用，会跳过第二轮测试

