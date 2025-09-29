# STREAM_DESIGN

本文档描述在 llm-connector 中为“多 Provider 可复用”的流式处理改造方案。设计目标：统一解析、统一错误与重试策略、降低 Provider 适配成本，并保持对各 Provider 特性的良好支持。

## 1. 背景与问题
- 当前部分 Provider 解析 SSE 基于逐行处理，遇到半包/多 `data:` 行聚合不稳、CRLF 差异等问题。
- 不同 Provider 的终止约定（`[DONE]`、`finish_reason`、连接关闭）不一致，导致上层消费逻辑复杂。
- 错误分类与重试逻辑分散在 Provider 内，重复代码多、可测试性弱。

## 2. 分层架构
- 传输层（HTTP 客户端）
  - 统一由 reqwest 构建，集中处理超时、代理、基础地址与公共 header 注入。
  - 针对本地 Mock/测试：当 `base_url` 指向 `localhost/127.0.0.1` 时禁用代理，避免系统代理导致 502。
- 流解析层（Parser 可插拔）
  - SSE 事件解析器：严格以双换行分隔事件、聚合多条 `data:` 行、兼容 CRLF、忽略 `[DONE]`，输出“完整事件 payload”。
  - NDJSON 解析器（第一阶段最小实现）：按行解析 NDJSON/JSON 对象流（每行一个 JSON），忽略空行。
  - 统一接口：返回 `Stream<Item = Result<String, LlmConnectorError>>`。
- 流编排层（可选增强，第一阶段不启用，feature flag 控制）
  - 第一阶段策略：默认关闭；不做“流级自动重试”，仅保留“请求级重试”（非 2xx/瞬时网络错误，带上限与指数退避）。
  - 预留能力：空闲超时（idle timeout）、早关闭重试、回退策略将在后续阶段按需开启，并通过 feature flag 控制范围。
- 事件到内部模型转换
  - Provider 层将自身事件 JSON 映射为统一的 `StreamingResponse/StreamingChoice/Delta`，工具调用与 reasoning 增量透传。
- 错误模型与重试判定
  - 统一使用 `LlmConnectorError`，`is_retryable()` 用于自动判定是否重试（网络、限流、Provider 错误可重试）。

## 3. 关键组件与接口
- `utils::streaming::sse_data_events(response: reqwest::Response) -> Stream<Result<String, LlmConnectorError>>`
  - 行为：
    - CRLF 归一化为 LF；按 `\n\n` 切分事件边界。
    - 聚合事件内多条 `data:` 为一个 payload（按规范用 `\n` 拼接）。
    - 忽略终止 `[DONE]` 事件。
- `utils::streaming::ndjson_events(response: reqwest::Response) -> Stream<Result<String, LlmConnectorError>>`（计划新增）
  - 行为：
    - 逐行读取，每行一个完整 JSON 事件；忽略空行。
- Provider 适配：
  - `chat_stream(&self, request) -> ChatStream`
    - 选择解析器（SSE/NDJSON），将每个 payload 反序列化为 Provider 自身的 `StreamResponse`，再转换为内部 `StreamingResponse`。

## 4. Provider 适配模式
- SSE Provider（如 DeepSeek）
  - 在 `chat_stream` 中使用 `sse_data_events`，确保事件完整性与多行聚合。
  - 在 `convert_stream_response` 中保持增量帧透传：
    - `delta.role`：仅在首帧出现。
    - `delta.content`：追加文本内容。
    - `delta.tool_calls` / `delta.reasoning_content`：按 Provider 行为透传或在上层聚合。
- NDJSON Provider（如部分 OpenAI Responses API）
  - 使用 `ndjson_events`；每行一个 JSON 帧，直接反序列化并转换为内部类型。

## 5. 流编排增强（可选，第一阶段不启用，feature flag 控制）
- 第一阶段策略：
  - 默认关闭流级自动重试与空闲超时；仅支持“请求级重试”（非 2xx / 网络瞬时错误），设置最大次数与指数退避。
- 预留能力（后续阶段按需开启）：
  - 空闲超时（idle timeout）
  - 早关闭重试（需明确完成信号约束以防重复计费）
  - 回退策略（固定延时或指数退避）
- 重试判定：
  - 使用 `LlmConnectorError::is_retryable()`（网络、限流、Provider 错误）。

## 6. 配置扩展（建议）
- 在 `ProviderConfig` 中新增：
  - `initial_request_max_retries: Option<u32>`（首次请求失败重试，第一阶段启用）
  - `stream_idle_timeout_ms: Option<u64>`（预留，默认禁用）
  - `stream_max_retries: Option<u32>`（预留，默认禁用）
- 第一阶段：只启用请求级重试相关配置；流编排相关配置作为预留，由 feature flag 控制开启。
- Provider 构造函数读取上述配置，或由编排层统一读取。

## 7. 测试策略
- 第一阶段最小集：
  - 多 `data:` 行聚合与事件边界（SSE），验证 CRLF 归一化、半包/粘包处理、忽略 `[DONE]` 终止帧。
  - NDJSON/JSON 对象行流：忽略空行、长行处理（不做流级重试）。
- 集成测试：
  - 至少 1 个真实 Provider 或本地兼容服务的端到端：请求到流的转化与消费（DeepSeek 优先）。
- 后续阶段（可选增强）：
  - 早关闭后重试成功；5xx 后重试成功；429 回退重试；空闲超时触发等。
- 集成测试：
  - CLI 流式输出与本地 fixture。
  - 提供者端到端：请求到流的转化与消费。

## 8. 迁移计划
- 第一步（当前里程碑）：
  - 将现有 SSE Provider 切换到 `sse_data_events`，保持 `convert_stream_response` 不变（DeepSeek 先行）。
  - 完成最小 `ndjson_events` 实现与一个示例 Provider 适配（逐行 JSON）。
  - 验收标准：端到端流稳定、事件聚合正确、CI 解析测试通过。
  - 回滚策略：若出现重大回归，快速回退到 `sse_data_lines`，并保留事件解析器的 feature flag 开关。
- 第二步：完善 NDJSON 适配覆盖与文档示例。
- 第三步（可选）：引入流编排包装（idle timeout/早关闭重试/回退策略），通过 feature flag 控制默认关闭。
- 第四步：补强测试与 CI，覆盖增强场景。

## 9. 风险与注意事项
- Provider 终止约定差异：
  - 有的使用 `[DONE]`，有的使用 `finish_reason`，也有直接连接关闭；上层消费者若不强制 Completed 事件，可在流结束时自然完成，否则在编排层补充终止事件或在最后帧补齐。
- 工具调用聚合：
  - 某些 Provider 以增量帧传递 `tool_calls.arguments`，需明确由哪一层聚合；建议默认“透传增量，按需在上层聚合”。
- 解析器可组合性：
  - SSE 与 NDJSON 混合或存在自定义控制帧的 Provider 需要额外适配点，解析器接口应保持简单稳定。

## 10. 新 Provider 接入清单
- 选择解析协议：SSE 或 NDJSON。
- 在 `chat_stream`：
  - 使用对应解析器（`sse_data_events`/`ndjson_events`）。
  - 反序列化 Provider 帧为自身类型，并转换为内部 `StreamingResponse`。
- 错误与重试：
  - 非 2xx：使用 `handle_error_response` 归一化为 `LlmConnectorError`。
  - 流式错误：依赖 `is_retryable()` 与编排层重试（如启用）。
- 测试：
  - Wiremock SSE/NDJSON 夹具；覆盖多行聚合、早关闭、5xx/429 等。

---

该方案的核心是通过“统一解析器 + 可选编排包装 + 标准化内部流式类型”实现多 Provider 复用与健壮性提升，并以最小改动逐步迁移现有 Provider，确保稳定性与可测试性。