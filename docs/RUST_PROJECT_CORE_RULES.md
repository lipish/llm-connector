# Rust 项目核心开发规范（速查表）

## 1. 命名规范

- **结构体/枚举**: `CamelCase` (如 `ChatRequest`, `LlmClient`)
- **函数/方法**: `snake_case` (如 `fetch_models`, `build_request_data`)
- **模块/文件**: `snake_case` (如 `client.rs`, `http_transport.rs`)
- **常量**: `SCREAMING_SNAKE_CASE` (如 `MAX_RETRY_ATTEMPTS`)
- **特性(trait)**: `CamelCase` (如 `ProviderAdapter`, `ErrorMapper`)
- **私有成员**: 可选使用下划线前缀 (如 `_internal_field`)

## 2. 组织规范 (v2.0架构)

- **核心优先**: 按功能重要性组织代码，核心功能优先
- **协议与Provider分离**:
  - `protocols/` 目录: 纯协议实现（OpenAI、Anthropic等官方API规范）
  - `providers/` 目录: Provider特定实现（Aliyun、Zhipu、Ollama等）
  - `core/` 目录: 核心trait定义（Protocol、Provider、HTTP传输等）
- **类型共享**: 公共请求/响应类型集中放在 `types/` 目录
- **错误统一**: 统一的错误处理模块 (`error.rs`)
- **文档位置**: 除README外，所有文档必须放在`docs/`目录下
- **测试位置**: 所有测试代码必须放在`tests/`目录下
- **最小暴露**: 只在 `lib.rs` 中重新导出公共API，内部实现保持私有
- **兼容性层**: 保持向后兼容的同时，提供清晰的v2.0 API

## 3. 接口设计 (v2.0)

- **协议与Provider分离**: Protocol定义API规范，Provider实现服务连接
- **命名约定**:
  - Protocol构造函数: `LlmClient::protocol_openai()`, `LlmClient::protocol_anthropic()`
  - Provider构造函数: `LlmClient::provider_aliyun()`, `LlmClient::provider_zhipu()`
  - 兼容性构造函数: `LlmClient::openai_compatible::<OpenAIProtocol>()`
- **向后兼容**: 保持v1.x API可用，添加deprecation警告
- **默认值模式**: 所有配置使用 `Default` trait 提供默认值
- **流式处理**: 支持同步和异步流式处理（通过feature flag控制）
- **无阻塞设计**: 异步IO操作优先使用非阻塞实现

## 4. 抽象模式 (v2.0)

- **Protocol模式**: 定义API规范接口，实现官方API标准
- **Provider模式**: 定义服务提供商接口，封装具体实现
- **兼容性层**: OpenAICompatibleProvider包装器，支持标准协议兼容
- **ErrorMapper模式**: 专用组件负责错误转换和处理
- **零成本抽象**: 优先使用不会产生运行时开销的抽象
- **Arc共享**: 使用 `Arc` 进行高效的资源共享（特别是配置和客户端）

## 5. 错误处理

- **专用错误类型**: 定义项目专用的错误类型
- **错误上下文**: 错误应包含足够上下文信息，便于调试
- **?操作符**: 使用 `?` 操作符简化错误处理流程
- **错误分类**: 区分可重试和不可重试的错误
- **格式化错误**: 使用 `format!` 或 `thiserror` 提供清晰的错误信息

## 6. 文档规范

- **模块文档**: 每个模块应有详细文档注释
- **API文档**: 所有公共API应有完整文档和示例
- **代码示例**: 示例代码应可运行且展示典型用例
- **设计决策**: 记录重要的设计决策和权衡

## 7. 测试规范

- **单元测试**: 为每个公共函数编写单元测试
- **文档测试**: 确保示例代码可运行（`///`注释中的代码块）
- **集成测试**: 测试组件间的交互
- **错误测试**: 测试各种错误情况和边界条件

## 8. 依赖管理

- **最小依赖**: 仅依赖必要的外部库
- **特性标志**: 使用feature flags控制可选功能
- **开发依赖**: 测试相关依赖放在`[dev-dependencies]`中
- **版本锁定**: 使用Cargo.lock确保构建一致性

## 9. 反模式（避免）

- **过度设计**: 避免过于复杂的抽象
- **硬编码值**: 避免硬编码配置和环境相关值
- **不必要的克隆**: 优化数据所有权和借用
- **过度使用unwrap**: 尽量不使用`unwrap()`处理可恢复错误
- **忽视生命周期**: 合理管理内存生命周期
- **协议污染**: 避免在纯协议实现中混入Provider特定逻辑
- **混合职责**: Protocol只处理API规范，Provider处理服务连接

## 10. 版本和兼容性 (v2.0)

- **语义化版本**: 遵循SemVer版本控制规范
- **向后兼容**: v2.0保持v1.x API可用，添加deprecation警告
- **迁移指南**: 版本升级时提供详细迁移说明（参见CLAUDE.md）
- **变更日志**: 维护详细的CHANGELOG文件
- **架构文档**: 重要架构变更记录在`docs/ARCHITECTURE.md`

---

*本规范基于llm-connector项目实践和Rust设计模式，可根据具体项目需求调整。*