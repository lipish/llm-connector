# QODAR_STREAM_DESIGN.md

# Qodar 流式处理设计方案

本文档描述基于现有 llm-connector 代码库分析后的流式处理重构方案，目标是支持多 Provider 的统一流式接口，同时保持代码的可维护性和扩展性。

## 1. 现状分析

### 当前实现优点
- 已有工作良好的 SSE 解析实现 (`sse_data_lines`, `sse_data_events`)
- DeepSeek Provider 基本可用
- 错误处理机制基本完善 (`LlmConnectorError::is_retryable()`)

### 存在问题
- 流处理逻辑与特定 Provider (OpenAI格式) 强耦合
- 缺少对不同协议 (NDJSON) 的支持
- 重试和超时配置硬编码
- 难以扩展到新的 Provider

## 2. 设计原则

- **渐进式改进**：在现有基础上优化，避免大规模重写
- **解耦分层**：解析器、处理器、重试逻辑分离
- **向后兼容**：确保现有 Provider 实现继续工作
- **配置驱动**：支持不同 Provider 的个性化配置

## 3. 架构设计

### 3.1 分层架构

```
┌─────────────────────────────────────┐
│          Provider Layer             │  ← Provider::chat_stream()
├─────────────────────────────────────┤
│        Stream Processor             │  ← StreamProcessor trait
├─────────────────────────────────────┤
│         Parser Layer                │  ← SSE/NDJSON/Custom parsers
├─────────────────────────────────────┤
│        Transport Layer              │  ← HTTP client (reqwest)
└─────────────────────────────────────┘
```

### 3.2 核心接口

#### StreamProcessor Trait
```rust
pub trait StreamProcessor: Send + Sync {
    /// 将 Provider 原始响应转换为统一格式
    fn convert_chunk(&self, raw_chunk: &str) -> Result<Option<StreamingResponse>, LlmConnectorError>;
    
    /// 检查是否为完成信号
    fn is_completion_signal(&self, raw_chunk: &str) -> bool;
    
    /// 获取流配置
    fn stream_config(&self) -> StreamConfig;
}
```

#### 解析器工厂
```rust
pub enum ParserType {
    SSE,
    NDJSON,
    Custom(Box<dyn Fn(reqwest::Response) -> ParseStream + Send + Sync>),
}

pub fn create_parser(response: reqwest::Response, parser_type: ParserType) -> ParseStream;
```

## 4. 实现方案

### 4.1 扩展配置支持

在 `ProviderConfig` 中添加流式配置：

```rust
pub struct ProviderConfig {
    // 现有字段
    pub api_key: String,
    pub base_url: Option<String>,
    pub timeout_ms: Option<u64>,
    
    // 新增流式配置
    pub stream_idle_timeout_ms: Option<u64>,
    pub stream_max_retries: Option<u32>,
    pub initial_request_max_retries: Option<u32>,
}
```

### 4.2 通用解析器扩展

在 `utils::streaming` 模块中添加：

```rust
// 保持现有的 sse_data_events 函数

// 新增 NDJSON 解析器
pub fn ndjson_events(response: reqwest::Response) -> ParseStream;

// 新增解析器工厂
pub fn create_parser(response: reqwest::Response, parser_type: ParserType) -> ParseStream;
```

### 4.3 通用流处理器

创建 `providers/streaming.rs` 模块：

```rust
pub async fn process_stream<P: StreamProcessor>(
    response: reqwest::Response,
    processor: P,
) -> Result<ChatStream, LlmConnectorError> {
    // 处理重试逻辑
    // 处理超时逻辑
    // 解析和转换响应
}
```

### 4.4 Provider 适配模式

每个 Provider 实现 `StreamProcessor`：

```rust
// DeepSeek 示例
struct DeepSeekStreamProcessor;

impl StreamProcessor for DeepSeekStreamProcessor {
    fn convert_chunk(&self, raw_chunk: &str) -> Result<Option<StreamingResponse>, LlmConnectorError> {
        // 将 DeepSeekStreamResponse 转换为 StreamingResponse
    }
    
    fn is_completion_signal(&self, raw_chunk: &str) -> bool {
        // 检查 "[DONE]" 或 finish_reason
    }
    
    fn stream_config(&self) -> StreamConfig {
        StreamConfig {
            parser_type: ParserType::SSE,
            idle_timeout_ms: 10000,
            max_retries: 3,
            completion_signals: vec![
                CompletionSignal::Marker("[DONE]".to_string()),
                CompletionSignal::JsonField {
                    field: "choices.0.finish_reason".to_string(),
                    value: serde_json::Value::String("stop".to_string()),
                },
            ],
        }
    }
}
```

## 5. 完成信号处理

### 支持的完成信号类型
- **标记信号**：如 `[DONE]` 字符串
- **JSON 字段**：如 `finish_reason: "stop"`
- **连接关闭**：流自然结束

### 配置示例
```rust
pub enum CompletionSignal {
    Marker(String),                           // "[DONE]"
    JsonField { field: String, value: Value }, // "choices.0.finish_reason" = "stop"
    ConnectionClosed,                         // 连接关闭
}
```

## 6. 重试策略

### 重试条件
- 网络错误 (`LlmConnectorError::NetworkError`)
- 限流错误 (`LlmConnectorError::RateLimitError`)
- Provider 错误 (`LlmConnectorError::ProviderError`)

### 重试逻辑
- **指数退避**：100ms, 200ms, 400ms, 800ms...
- **最大延时**：5秒
- **最大重试次数**：可配置，默认 3 次

### 重试场景
1. **初始请求失败**：完全重新发起请求
2. **流中断**：检测到空闲超时或早关闭时重试
3. **解析错误**：部分 JSON 解析错误可重试

## 7. 迁移策略

### 第一阶段：基础架构
1. 扩展 `ProviderConfig` 结构
2. 实现 `ndjson_events` 解析器
3. 创建 `StreamProcessor` trait 和通用处理器

### 第二阶段：Provider 迁移
1. 将 DeepSeek Provider 迁移到新架构
2. 保持现有接口不变，内部使用新的处理逻辑
3. 添加配置支持

### 第三阶段：增强功能
1. 实现完整的重试逻辑
2. 添加更多完成信号类型支持
3. 性能优化和监控

### 第四阶段：扩展支持
1. 为新 Provider 提供模板和示例
2. 添加自定义解析器支持
3. 完善测试覆盖

## 8. 测试策略

### 单元测试
- 解析器测试：SSE/NDJSON 格式处理
- 转换器测试：Provider 响应到内部格式转换
- 重试逻辑测试：各种错误场景

### 集成测试
- 真实 Provider 测试：实际 API 调用验证
- 网络异常测试：超时、连接中断场景
- 端到端测试：完整流式对话流程

### 性能测试
- 大流量处理能力
- 内存使用效率
- 重试机制性能影响

## 9. Provider 接入指南

### 新 Provider 接入步骤

1. **实现 StreamProcessor**
```rust
struct NewProviderStreamProcessor {
    config: ProviderConfig,
}

impl StreamProcessor for NewProviderStreamProcessor {
    // 实现必要方法
}
```

2. **配置解析器类型**
```rust
fn stream_config(&self) -> StreamConfig {
    StreamConfig {
        parser_type: ParserType::SSE, // 或 NDJSON
        idle_timeout_ms: self.config.stream_idle_timeout_ms.unwrap_or(10000),
        // ...
    }
}
```

3. **定义完成信号**
```rust
completion_signals: vec![
    CompletionSignal::Marker("[DONE]".to_string()),
    // 或其他信号类型
],
```

### 最佳实践
- **增量转换**：尽可能保持原始数据结构，只转换必要字段
- **错误处理**：区分可重试和不可重试错误
- **配置驱动**：避免硬编码，使用配置参数
- **日志记录**：添加适当的调试和错误日志

## 10. 与原设计的区别

### 相比 STREAM_DESIGN.md 的改进
1. **基于现有代码**：利用已有的 `sse_data_events` 实现
2. **渐进式迁移**：避免大规模重写风险
3. **简化重试逻辑**：专注于明确的网络错误，避免复杂状态管理
4. **保持兼容性**：现有 Provider 实现无需立即修改

### 降低的复杂度
- 移除了复杂的流编排层概念
- 简化了错误分类和重试判定
- 避免了过度抽象的 Stream Orchestrator

## 11. 风险评估

### 技术风险
- **接口稳定性**：新 trait 设计需要经过充分验证
- **性能影响**：额外的抽象层可能影响性能
- **维护复杂度**：需要维护多个解析器和处理器

### 缓解措施
- **分阶段实施**：逐步验证每个组件
- **性能基准**：建立性能基线和回归测试
- **文档完善**：提供详细的接入指南和示例

## 12. 成功指标

- **接入效率**：新 Provider 接入时间从数天减少到数小时
- **代码复用**：解析和重试逻辑代码复用率 > 80%
- **稳定性**：流式连接成功率 > 99%
- **性能**：相比现有实现性能损失 < 5%

---

本设计方案在保持系统稳定性的前提下，为 llm-connector 提供了统一、可扩展的流式处理架构，为支持更多 LLM Provider 奠定了坚实基础。