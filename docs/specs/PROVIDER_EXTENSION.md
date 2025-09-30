# Provider Extension Guide

如何为 llm-connector 添加新的 LLM 提供商支持。

## 设计原则

- **Adapter 模式**：使用 ProviderAdapter trait 实现协议转换
- **GenericProvider 模板**：所有 Provider 使用统一的 GenericProvider<T> 实现
- **不创建新文件**：所有 Adapter 在 `src/providers/adapters.rs` 中实现
- **协议转换**：将提供商 API 转换为 OpenAI 兼容格式
- **错误映射**：统一错误处理和重试策略
- **配置一致**：使用相同的配置模式

## 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                     Application                          │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│         GenericProvider<NewProviderAdapter>              │
│                  (统一实现，无需修改)                      │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│              NewProviderAdapter                          │
│           (唯一需要实现的代码)                            │
│    - build_request_data()                                │
│    - parse_response_data()                               │
│    - parse_stream_response_data()                        │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                  HttpTransport                           │
│              (统一 HTTP 传输层)                           │
└─────────────────────────────────────────────────────────┘
```

## Streaming 适配规范

**重要**：llm-connector 仅支持 SSE（Server-Sent Events）协议，因为所有主流商业 LLM Provider 都使用 SSE。

- **解析器**：使用 `utils::streaming::sse_data_events`
  - 严格按事件边界聚合多 data 行
  - CRLF 归一化
  - 忽略 "[DONE]" 完成帧
  
- **终止语义**：
  - 连接正常结束 = 完成
  - 如提供商提供 `finish_reason`，在最后帧进行标记
  
- **转换规则**：
  - 将 Provider 原始帧转换为 StreamingResponse/StreamingChoice/Delta
  - delta.role 仅首帧
  - delta.content 累积追加
  - tool_calls.arguments 增量透传
  - usage 仅可能在最后帧
  
- **测试要求**：
  - 覆盖多 data 行聚合、CRLF 归一化、半包/粘包
  - 忽略 "[DONE]"
  - 至少 1 个真实或本地兼容 Provider 端到端用例

## 新增 Provider 只需 3 步

### 步骤 1：在 `src/providers/adapters.rs` 中添加 Adapter

**重要**：不需要创建新文件！所有 Adapter 都在 `adapters.rs` 中实现（~150 行代码）。

参考 `DeepSeekAdapter`、`AliyunAdapter`、`ZhipuAdapter` 的实现。

### 步骤 2：在 `src/providers/mod.rs` 中 re-export

```rust
pub use adapters::NewProviderAdapter;
```

### 步骤 3：使用新 Provider

```rust
// 方式 1：直接使用
let provider = GenericProvider::new(config, NewProviderAdapter)?;

// 方式 2：通过 Registry（推荐）
registry.register("new_provider", config, NewProviderAdapter)?;
```

## 详细实现指南

请参考 `docs/ARCHITECTURE_DESIGN.md` 中的 "Provider 架构设计" 章节，包含完整的代码示例和最佳实践。

## 总结

- ✅ **不需要创建新文件**
- ✅ 代码复用率达到 **85%**
- ✅ 新增 Provider 只需 ~150 行代码
- ✅ 所有 Provider 使用统一的 GenericProvider 实现
