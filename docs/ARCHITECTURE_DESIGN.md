# ARCHITECTURE_DESIGN

面向 llm-connector 的分阶段架构与推进方案。本文件已融合并整合 docs/STREAM_DESIGN.md（可立即落地的基线优化）与 docs/QODAR_STREAM_DESIGN.md（扩展蓝图），采用“两阶段渐进式”实现路径：第一阶段专注统一解析与低风险改造，第二阶段在 feature flag 控制下逐步引入通用抽象与编排能力。

## 0. 合并说明与关联文档
- 本文件为合并版架构设计：以 STREAM_DESIGN 为第一阶段基线，以 QODAR_STREAM_DESIGN 为第二阶段参考。
- 仍保留原文档：STREAM_DESIGN.md（细节基线与迁移计划）、QODAR_STREAM_DESIGN.md（抽象/编排蓝图与示例）。

## 1. 设计目标
- 统一与健壮：事件级解析统一（SSE/NDJSON），解决半包、多 data 行、CRLF 兼容等问题。
- 向后兼容：阶段一仅替换解析器，不改变上层类型与消费路径。
- 风险可控：禁用“流级自动重试”，仅保留“请求级重试”，避免重复计费与状态不一致。
- 可扩展：阶段二引入可插拔解析器工厂、完成信号抽象、编排器（idle/早关闭重试/退避），以 feature flag 灰度上线。

## 2. 分层架构概览
- Transport 层：reqwest HTTP 客户端（超时、代理、基础地址与公共 headers），本地测试场景禁用系统代理。
- Parser 层：
  - SSE 事件解析器：按事件边界聚合多条 data 行，CRLF 归一化，忽略 `[DONE]` 完成帧。
  - NDJSON 行解析器：第一阶段最小实现（逐行 JSON，忽略空行）。
- Provider 层：将解析得到的原始帧映射为统一内部流式类型（StreamingResponse/Delta）。
- Orchestrator 层（可选增强，阶段二）：空闲超时、早关闭重试、退避策略；默认关闭，feature flag 控制。

---

## 3. 阶段一：基线落地（统一解析 + 最小 NDJSON + 禁用流级重试）

### 3.1 范围与原则
- 将现有 SSE Provider 切换到事件级解析器 `utils::streaming::sse_data_events`（DeepSeek 先行）。
- 新增最小 `utils::streaming::ndjson_events`：逐行 JSON 对象流，忽略空行，不做流级重试。
- 保留“请求级重试”：仅针对初始请求失败（非 2xx / 网络瞬时错误），设最大次数与指数退避；禁用“流级自动重试/空闲超时”。

### 3.2 关键行为与接口
- sse_data_events(response: reqwest::Response) -> Stream<Result<String, LlmConnectorError>>
  - CRLF 归一化为 LF，按 `\n\n` 切分事件边界。
  - 聚合事件内所有 `data:` 行为一个 payload（用 `\n` 连接）。
  - 忽略 `[DONE]` 终止事件，由上层以“连接正常结束=完成”处理。
- ndjson_events(response: reqwest::Response) -> Stream<Result<String, LlmConnectorError>>（新增最小实现）
  - 逐行读取；每行一个完整 JSON；忽略空行与仅空白行。
- Provider 适配：
  - `chat_stream(&self, request) -> ChatStream`：选择解析器（SSE/NDJSON），将每个 payload 反序列化为 Provider 自身的 StreamResponse，再转换为内部 StreamingResponse/Delta。

#### 3.2.1 解析器健壮性与错误分类（阶段一实施）
- 错误子类建议：
  - TransportReadError（网络/IO 读取错误，可重试）
  - IncompleteChunk（半包/截断，不可重试）
  - InvalidSseEvent（SSE 事件格式错误，不可重试）
  - MalformedJson（畸形 JSON，不可重试）
  - EncodingError（编码错误，默认不可重试）
- 资源与安全阈值：
  - 最大事件大小与最大行长度限制（防止内存过度占用），超限直接判为 InvalidSseEvent/MalformedJson。
  - UTF-8 严格解码为默认策略，保留“宽容模式（lossy）”的可选开关，默认关闭。
- 测试覆盖点：
  - 半包/粘包、CRLF 与 BOM 处理、空行与仅空白行、巨长行与超限事件、畸形 JSON 行、多 data 行聚合、忽略 `[DONE]` 的回归测试。


### 3.3 配置
- ProviderConfig 新增/启用：
  - `initial_request_max_retries: Option<u32>`：首次请求失败重试（阶段一启用）。
  - 预留 `stream_idle_timeout_ms`、`stream_max_retries`（阶段一禁用，阶段二由 feature flag 控制）。

### 3.4 测试与验收
- 单元测试（解析）：
  - SSE：多 `data:` 行聚合、CRLF 归一化、半包/粘包、忽略 `[DONE]`。
  - NDJSON：逐行 JSON、忽略空行与异常行（最小实现）。
- 集成测试：至少 1 个真实 Provider 或本地兼容服务（DeepSeek 优先）。
- 验收标准：端到端流稳定；SSE/NDJSON 解析测试通过；示例运行与 CI 通过。
- 回滚策略：保留对 `sse_data_lines` 的快速回退路径；解析器选择支持开关切换。

### 3.5 按文件实施清单
- src/providers/deepseek.rs：将 `chat_stream` 的解析入口从 `sse_data_lines` 切换到 `sse_data_events`；保持 `convert_stream_response` 与上层类型不变。
- src/utils/mod.rs：新增 `ndjson_events(response) -> Stream<Result<String, LlmConnectorError>>` 及最小测试夹具。
- docs：STREAM_DESIGN.md 已作为阶段一基线与迁移计划。

---

## 4. 阶段二：扩展增强（抽象 + 编排，灰度上线）

### 4.1 新增核心抽象
- 解析器工厂（Parser Factory）：
  - `create_parser(response, parser_type: ParserType) -> ParseStream`
  - ParserType：SSE / NDJSON / Custom(Box<dyn Fn(reqwest::Response) -> ParseStream>)。
- 完成信号抽象（CompletionSignal）：
  - Marker("[DONE]")、JsonField{ field: String, value: Value }、ConnectionClosed。
- StreamProcessor trait（统一转换接口）：
  - `convert_chunk(&self, raw_chunk: &str) -> Result<Option<StreamingResponse>, LlmConnectorError>`
  - `is_completion_signal(&self, raw_chunk: &str) -> bool`
  - `stream_config(&self) -> StreamConfig`
- Orchestrator（可选）：
  - 空闲超时（idle timeout）、早关闭重试、指数退避；默认关闭，通过 feature flag 在小范围试点。

#### 4.1.1 Provider 抽象统一性约束
- 语义映射：遵循 OpenAI 兼容基线——delta.role 仅首帧、delta.content 累积、tool_calls.arguments 增量透传、usage 仅可能末帧；禁止在中间帧输出 usage。
- 错误收敛：将提供商特有错误映射到统一错误族（Authentication/RateLimit/Provider/Streaming/Network），用于 is_retryable 判定；流式解析错误默认不重试。
- 实现约束：Provider 的 convert_stream_response 应只做语义转换，不引入状态机副作用；完成信号识别在阶段二通过 is_completion_signal 实现。
- 测试约束：最少包含 SSE/NDJSON 正常与边界用例；对齐 PROVIDER_EXTENSION.md 的适配与测试要求。
- 文档交叉：在具体 Provider 文档或实现注释中链接到本章节与 PROVIDER_EXTENSION.md。

### 4.2 配置与策略
- ProviderConfig：
  - 阶段二在灰度范围内开启 `stream_idle_timeout_ms`、`stream_max_retries`，并与 Orchestrator 集成。
- 重试策略（仅阶段二启用在灰度）：
  - 指数退避（100ms, 200ms, 400ms, …，最大 5s）；上限可配置。
  - 适用条件：网络错误、限流、Provider 可重试错误；避免对解析增量重复计费。

#### 4.2.1 Feature Flag 分层与热更新
- 层级覆盖：Global → Provider → Model → User/Tenant，采用就近覆盖解析。
- 典型 flags：`stream_orchestrator`、`stream_idle_timeout`、`stream_retry`、`completion_signal_json`。
- 热更新：提供 Client 级 set_feature_flags(...) 或共享可原子替换的标志结构（Arc/原子 Swap），确保无需重启即可生效。
- 容错与类型安全：阶段一保持 `feature_flags: Option<Vec<String>>`；阶段二建议引入枚举/位域，减少拼写错误与解析失败。
### 4.3 迁移与试点
- 选择 1 个 Provider 迁移到 StreamProcessor + Parser Factory 架构，验证完成信号抽象与编排策略对稳定性与性能影响（< 5%）。
- 保持对外接口与内部类型不变；仅替换内部实现。

### 4.4 测试与观测
- 单元测试：解析器工厂、完成信号识别、处理器转换。
- 集成测试：空闲超时、早关闭重试、5xx/429 回退重试；端到端对话流程。
- 性能与观测：事件计数、重试次数、超时触发、性能回归基线与监控。

---

## 5. 风险与边界
- 重试风险：阶段一禁用流级自动重试，避免重复计费与状态不一致；阶段二以 feature flag 灰度开启并度量影响。
- 终止语义差异：SSE `[DONE]`、JSON `finish_reason`、连接关闭的差异通过 CompletionSignal 抽象在阶段二统一；阶段一采用保守策略（连接正常结束=完成）。
- 兼容性：不改变内部流式类型与 Provider 的 `convert_stream_response` 行为；解析器替换为透明优化。

## 5.5 观测与性能基线（阶段一先埋点，阶段二完善）
- 指标建议：连接延迟（connect→first byte）、解析耗时（chunk_parse_ms）、总帧数、总耗时（first byte→completion）、错误率（按子类）、重试次数与超时事件（阶段二）。
- 埋点机制：提供 TelemetrySink trait（默认 no-op），或可选集成 tracing；为每次请求生成 correlation_id 串联日志与指标。
- 文档同步：在实现处加入注释与章节引用，确保观测埋点作为阶段一完成后的基线。

## 5.6 文档-代码同步机制
- 注释约定：在关键实现处以章节号注释（如“参见 ARCHITECTURE_DESIGN §3.2.1”），便于跨文件检索。
- 示例约定：在 PROVIDER_EXTENSION.md 提供 SSE/NDJSON 片段示例，与 API.md 的 Streaming 行为描述一致。
- 自动化校验：在 CI 增加 docs/specs 与 src 的关键术语与字段一致性检查（术语：delta.role/content/tool_calls.arguments/usage；字段：parser_type/feature_flags/initial_request_max_retries）。
- 变更流程：当解析器或流语义发生调整，必须同步更新 API.md/ERRORS.md/CONFIG_SCHEMA.md/PROVIDER_EXTENSION.md 与 ARCHITECTURE_DESIGN.md，并在 PR 模板中列出链接与变更摘要。

## 6. 成功指标
- 新 Provider 接入效率提升（从数天降至数小时）。
- 解析与重试逻辑复用率 > 80%。
- 流式连接成功率 > 99%。
- 阶段二性能回归 < 5%。

## 7. 里程碑与排期（建议）
- M1（1-2 天）：完成解析器替换与最小 NDJSON，实现与测试通过；更新文档与示例。
- M2（3-5 天）：在一个 Provider 试点阶段二（trait/工厂/CompletionSignal/编排），建立灰度策略与性能基线。

---

本合并方案以“解析层统一 + 向后兼容 + 可选编排”的路径，在低风险前提下提升工程稳定性与扩展能力：第一阶段专注于可立即落地的改进；第二阶段以 feature flag 方式逐步引入更强抽象与编排设施，便于在真实场景中验证后再推广。