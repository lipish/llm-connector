# PROVIDER_DESIGN.md

## 1. 当前问题分析
现有 Provider 实现（如 `deepseek.rs`、`aliyun.rs`）采用独立罗列方式，存在以下问题：
- **代码重复**：每个 Provider 重复实现 HTTP 客户端构建、错误处理、请求/响应转换逻辑。
- **维护成本高**：新增 Provider 需复制粘贴大量模板代码，易引入不一致性。
- **扩展性差**：硬编码的解析逻辑难以适配新协议（如 NDJSON、自定义流格式）。
- **配置分散**：每个 Provider 自行处理超时、重试等配置，缺乏统一管理。

## 2. 设计目标
- **统一抽象**：提取公共逻辑（HTTP 传输、错误处理、配置管理）到基础层。
- **协议无关性**：通过可插拔解析器支持多种流式协议（SSE、NDJSON、自定义）。
- **配置驱动**：使用统一配置结构定义 Provider 行为，减少代码重复。
- **向后兼容**：保持现有 Provider 接口不变，仅重构内部实现。
- **阶段对齐**：与架构设计的阶段二（抽象与编排）平滑集成。

## 3. 核心抽象设计

### 3.1 基础 Provider Trait 增强
扩展现有 `Provider` trait，增加生命周期和泛型支持，以统一异步流处理：
```rust
#[async_trait]
pub trait Provider: Send + Sync {
    // 现有方法保持不变
    fn name(&self) -> &str;
    fn supported_models(&self) -> Vec<String>;
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;
    
    // 增强流式方法，支持泛型解析器
    #[cfg(feature = "streaming")]
    async fn chat_stream(
        &self,
        request: &ChatRequest,
    ) -> Result<BoxStream<'static, Result<StreamingResponse, LlmConnectorError>>, LlmConnectorError>;
}
```

### 3.2 统一 HTTP 传输层
抽象 HTTP 客户端操作，处理重试、超时、代理等公共逻辑：
```rust
pub struct HttpTransport {
    client: Client,
    config: HttpConfig,
}

impl HttpTransport {
    pub async fn send_request(
        &self,
        request: HttpRequest,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        // 统一处理请求重试、错误映射、日志等
    }
    
    pub fn build_client(config: &HttpConfig) -> Result<Client, LlmConnectorError> {
        // 统一客户端构建逻辑（超时、代理禁用等）
    }
}
```

### 3.3 可插拔解析器工厂
与架构设计阶段二对齐，引入解析器工厂支持多种协议：
```rust
pub enum ParserType {
    Sse,
    Ndjson,
    Custom(Box<dyn Fn(reqwest::Response) -> ParseStream>),
}

pub trait ParserFactory: Send + Sync {
    fn create_parser(
        &self,
        response: reqwest::Response,
        parser_type: ParserType,
    ) -> BoxStream<'static, Result<String, LlmConnectorError>>;
}

// 默认实现基于 utils::streaming 模块
pub struct DefaultParserFactory;
impl ParserFactory for DefaultParserFactory {
    fn create_parser(...) { ... } // 使用 sse_data_events 或 ndjson_events
}
```

### 3.4 配置驱动的 Provider 模板
定义通用 Provider 模板，通过配置减少重复代码：
```rust
pub struct GenericProvider<C, P> {
    config: C,      // Provider 特定配置
    transport: HttpTransport,
    parser_factory: P, // 解析器工厂
}

impl<C, P> GenericProvider<C, P>
where
    C: ProviderConfig,
    P: ParserFactory,
{
    pub fn new(config: C) -> Result<Self, LlmConnectorError> {
        // 通用初始化逻辑
    }
    
    // 实现 chat 和 chat_stream 的通用版本
    async fn chat_generic(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // 使用 transport 发送请求，使用 config 映射错误
    }
}
```

## 4. 具体实施建议

### 4.1 阶段一：提取公共模块（立即可行）
1. **创建 `src/providers/common/` 模块**：
   - `http_transport.rs`：统一 HTTP 客户端构建与请求发送。
   - `error_mapper.rs`：定义错误映射 trait，各 Provider 实现特定错误转换。
   - `config.rs`：扩展 `ProviderConfig`，支持解析器类型、重试策略等。

2. **重构现有 Provider**：
   - 将 `deepseek.rs` 和 `aliyun.rs` 中的公共逻辑移至 `common` 模块。
   - 保持对外接口不变，仅内部使用共享组件。

### 4.2 阶段二：引入抽象模板（与架构设计同步）
1. **实现 `GenericProvider` 模板**：
   - 为新增 Provider 提供基础实现，只需配置差异部分。
   - 集成解析器工厂，支持 SSE/NDJSON 自动切换。

2. **迁移示例**：
   - 新 Provider（如 Kimi、GLM）直接基于 `GenericProvider` 实现。
   - 现有 Provider 逐步迁移，通过 feature flag 控制。

### 4.3 配置优化
统一配置结构，支持声明式 Provider 定义：
```rust
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout_ms: Option<u64>,
    // 新增通用配置
    pub parser_type: ParserType, // SSE 或 NDJSON
    pub max_retries: Option<u32>,
    pub retry_backoff_ms: Option<u64>,
}
```

## 5. 优势与收益
- **代码复用率提升**：新 Provider 实现代码量减少 70%+（仅需配置映射逻辑）。
- **维护简便**：错误处理、HTTP 客户端等逻辑单一维护点。
- **协议扩展性**：新增流协议只需实现 `ParserFactory`，无需修改 Provider。
- **测试覆盖**：公共逻辑可统一测试，提升质量。

## 6. 实施路线图
1. **本周**：提取 HTTP 传输层和错误映射模块，重构 DeepSeek Provider 作为参考。
2. **下周**：实现 `GenericProvider` 模板，用于新 Provider 开发。
3. **下月**：与架构设计阶段二集成，引入解析器工厂和编排器。

## 7. 高级优化思路

### 7.1 基于宏的 Provider 声明式定义
使用过程宏自动生成 Provider 模板代码，大幅减少样板代码：
```rust
#[derive(Provider)]
#[provider(name = "deepseek", base_url = "https://api.deepseek.com")]
#[supports_models("deepseek-chat", "deepseek-reasoner")]
pub struct DeepSeekConfig {
    api_key: String,
    timeout_ms: Option<u64>,
}

// 宏自动生成：
// - Provider trait 实现
// - 请求/响应转换方法
// - 错误映射逻辑
```

### 7.2 响应式配置热更新
支持运行时配置动态调整，无需重启服务：
```rust
pub struct DynamicProvider<C, P> {
    config: Arc<RwLock<C>>,
    transport: HttpTransport,
    parser_factory: P,
}

impl<C, P> DynamicProvider<C, P> {
    pub fn update_config(&self, new_config: C) {
        *self.config.write().unwrap() = new_config;
    }
    
    pub fn watch_config_changes(&self, config_source: impl Stream<Item = C>) {
        // 监听配置变化并自动更新
    }
}
```

### 7.3 智能路由与负载均衡
对于支持多地域/多端点的 Provider，实现智能路由：
```rust
pub struct MultiEndpointProvider {
    endpoints: Vec<EndpointConfig>,
    health_checker: HealthChecker,
    load_balancer: LoadBalancer,
}

impl MultiEndpointProvider {
    pub async fn select_endpoint(&self) -> Result<&EndpointConfig, LlmConnectorError> {
        // 基于健康状态、延迟、负载等选择最优端点
    }
}
```

### 7.4 可观测性集成
内置监控指标和追踪支持：
```rust
pub struct InstrumentedProvider<P> {
    inner: P,
    metrics: ProviderMetrics,
    tracer: Tracer,
}

impl<P: Provider> Provider for InstrumentedProvider<P> {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let span = self.tracer.start_span("provider_chat");
        let _guard = span.enter();
        
        let start = Instant::now();
        let result = self.inner.chat(request).await;
        let duration = start.elapsed();
        
        self.metrics.record_request(duration, result.is_ok());
        result
    }
}
```

### 7.5 缓存策略抽象
统一缓存层，支持多级缓存（内存、Redis等）：
```rust
pub trait ResponseCache: Send + Sync {
    async fn get(&self, key: &CacheKey) -> Option<CachedResponse>;
    async fn set(&self, key: CacheKey, response: CachedResponse, ttl: Duration);
}

pub struct CachingProvider<P, C> {
    inner: P,
    cache: C,
    cache_strategy: CacheStrategy,
}
```

### 7.6 容错与降级机制
实现更精细的故障处理策略：
```rust
pub enum FallbackStrategy {
    RetryWithBackoff,    // 指数退避重试
    SwitchEndpoint,      // 切换端点
    DegradeToSimplerModel, // 降级到简单模型
    ReturnCachedResult,  // 返回缓存结果
    Custom(Box<dyn Fn() -> FallbackAction>),
}

pub struct ResilientProvider<P> {
    inner: P,
    fallback_strategies: Vec<FallbackStrategy>,
    circuit_breaker: CircuitBreaker,
}
```

## 8. 实施优先级建议

**高优先级（立即实施）**：
1. 宏声明式定义 - 最大程度减少重复代码
2. 统一 HTTP 传输层 - 基础架构改进

**中优先级（下个版本）**：
3. 可观测性集成 - 提升运维能力
4. 响应式配置 - 增强灵活性

**低优先级（未来规划）**：
5. 智能路由与负载均衡
6. 缓存策略抽象
7. 高级容错机制

## 9. 参考文档
- [ARCHITECTURE_DESIGN.md](./ARCHITECTURE_DESIGN.md)：整体架构与阶段规划。
- 现有 Provider 代码：`src/providers/deepseek.rs`、`src/providers/aliyun.rs`。

此设计通过分层抽象和配置驱动，在保持兼容性的同时显著提升工程效率。建议从阶段一开始渐进实施，逐步验证效果。高级优化思路为长期演进提供了清晰的技术路线。