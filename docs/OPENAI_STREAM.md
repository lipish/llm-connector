## Codex OpenAI Stream Chat处理机制分析

### 整体架构

Codex 支持两种不同的 OpenAI API 协议：

1. **Chat Completions API** (`WireApi::Chat`) - 标准的 `/v1/chat/completions` 端点
2. **Responses API** (`WireApi::Responses`) - OpenAI 的实验性 `/v1/responses` 端点

### 核心组件

#### 1. ModelClient 流处理调度

在 `ModelClient::stream()` 方法中，根据 provider 配置决定使用哪种协议：

```rust
pub async fn stream(&self, prompt: &Prompt) -> Result<ResponseStream> {
    match self.provider.wire_api {
        WireApi::Responses => self.stream_responses(prompt).await,
        WireApi::Chat => {
            // 创建原始流连接
            let response_stream = stream_chat_completions(
                prompt,
                &self.config.model_family,
                &self.client,
                &self.provider,
            ).await?;

            // 使用聚合适配器处理流数据
            let mut aggregated = if self.config.show_raw_agent_reasoning {
                AggregatedChatStream::streaming_mode(response_stream)
            } else {
                response_stream.aggregate()
            };

            // 通过 channel 桥接聚合流
            let (tx, rx) = mpsc::channel::<Result<ResponseEvent>>(16);
            tokio::spawn(async move {
                while let Some(ev) = aggregated.next().await {
                    if tx.send(ev).await.is_err() { break; }
                }
            });

            Ok(ResponseStream { rx_event: rx })
        }
    }
}
```

#### 2. Chat Completions SSE 处理流程

##### 请求构建 (`stream_chat_completions`)

1. **消息格式化**：将 Codex 内部的 `ResponseItem` 转换为 OpenAI Chat Completions 格式
2. **推理内容处理**：将 `Reasoning` 项附加到相应的 assistant 消息中
3. **工具调用处理**：支持 function calls 和 local shell calls
4. **重试机制**：带有指数退避的请求重试

##### SSE 流处理 (`process_chat_sse`)

核心的 SSE 处理函数处理以下事件类型：

1. **文本增量** (`OutputTextDelta`):
```rust
if let Some(content) = choice
    .get("delta")
    .and_then(|d| d.get("content"))
    .and_then(|c| c.as_str())
    && !content.is_empty()
{
    assistant_text.push_str(content);
    let _ = tx_event
        .send(Ok(ResponseEvent::OutputTextDelta(content.to_string())))
        .await;
}
```

2. **推理内容增量** (`ReasoningContentDelta`):
```rust
if let Some(reasoning_val) = choice.get("delta").and_then(|d| d.get("reasoning")) {
    let mut maybe_text = reasoning_val
        .as_str()
        .map(str::to_string)
        .filter(|s| !s.is_empty());

    // 处理对象格式的推理内容 { "text": "..." } 或 { "content": "..." }
    if maybe_text.is_none() && reasoning_val.is_object() {
        // ... 解析逻辑
    }

    if let Some(reasoning) = maybe_text {
        reasoning_text.push_str(&reasoning);
        let _ = tx_event
            .send(Ok(ResponseEvent::ReasoningContentDelta(reasoning)))
            .await;
    }
}
```

3. **工具调用处理**：
```rust
if let Some(tool_calls) = choice
    .get("delta")
    .and_then(|d| d.get("tool_calls"))
    .and_then(|tc| tc.as_array())
    && let Some(tool_call) = tool_calls.first()
{
    fn_call_state.active = true;
    // 收集 call_id, function name, arguments
}
```

4. **完成处理**：
```rust
if let Some(finish_reason) = choice.get("finish_reason").and_then(|v| v.as_str()) {
    match finish_reason {
        "tool_calls" => {
            // 发送 FunctionCall 事件
        }
        "stop" => {
            // 发送最终的 Message 事件
        }
        _ => {}
    }
}
```

#### 3. 流聚合机制 (`AggregatedChatStream`)

Codex 提供了两种流处理模式：

1. **AggregatedOnly 模式**：只发送最终聚合的消息
2. **Streaming 模式**：发送增量更新和最终消息

```rust
enum AggregateMode {
    AggregatedOnly,  // 仅最终消息
    Streaming,       // 流式 + 最终消息
}
```

#### 4. 事件类型 (`ResponseEvent`)

```rust
pub enum ResponseEvent {
    Created,
    OutputItemDone(ResponseItem),
    Completed { response_id: String, token_usage: Option<TokenUsage> },
    OutputTextDelta(String),              // 文本增量
    ReasoningSummaryDelta(String),        // 推理摘要增量
    ReasoningContentDelta(String),        // 推理内容增量
    ReasoningSummaryPartAdded,
    WebSearchCallBegin { call_id: String },
    RateLimits(RateLimitSnapshot),
}
```

### 测试覆盖

从 [chat_completions_sse.rs](file:///Users/mac-m4/github/codex/codex-rs/core/tests/chat_completions_sse.rs) 测试文件可以看到，Codex 测试了以下场景：

1. **基本文本流**：无推理内容的简单文本响应
2. **字符串推理**：推理作为简单字符串的流处理
3. **对象推理**：推理作为嵌套对象的流处理  
4. **最终消息推理**：推理仅在最终消息中提供
5. **工具调用前推理**：在工具调用之前的推理处理

### 关键特性

1. **多协议支持**：同时支持 Chat Completions 和 Responses API
2. **推理处理**：完整支持 OpenAI 的推理/思考过程流
3. **工具调用**：支持 function calls 和 local shell calls
4. **流聚合**：可配置的流聚合，支持实时流和最终聚合
5. **重试机制**：带指数退避的错误重试
6. **超时处理**：可配置的流空闲超时

这个实现非常全面，不仅处理了标准的 OpenAI Chat Completions SSE 流，还支持了推理过程、工具调用等高级功能，同时提供了灵活的聚合选项以适应不同的使用场景。