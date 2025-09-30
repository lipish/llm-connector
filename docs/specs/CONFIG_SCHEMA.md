# Configuration Schema

llm-connector 配置规范，专注于简单的提供商配置。

## 设计原则

- **简单优先**：最小化配置复杂度
- **环境变量优先**：推荐使用环境变量而非配置文件
- **无状态**：不包含路由、负载均衡等状态管理
- **提供商平等**：所有提供商使用相同的配置模式

## 配置方式

### 1. 环境变量（推荐）

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."
export OPENAI_BASE_URL="https://api.openai.com/v1"  # 可选
export OPENAI_TIMEOUT_MS="30000"                    # 可选

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_BASE_URL="https://api.anthropic.com"  # 可选
export ANTHROPIC_TIMEOUT_MS="30000"                    # 可选

# DeepSeek
export DEEPSEEK_API_KEY="sk-..."
export DEEPSEEK_BASE_URL="https://api.deepseek.com/v1"  # 可选
export DEEPSEEK_TIMEOUT_MS="30000"                      # 可选

# GLM (智谱)
export GLM_API_KEY="..."
# 或者
export ZHIPU_API_KEY="..."
export GLM_BASE_URL="https://open.bigmodel.cn/api/paas/v4"  # 可选
export GLM_TIMEOUT_MS="30000"                               # 可选

# Qwen (阿里)
export QWEN_API_KEY="..."
# 或者
export ALIBABA_QWEN_API_KEY="..."
export QWEN_BASE_URL="https://dashscope.aliyuncs.com/compatible-mode/v1"  # 可选
export QWEN_TIMEOUT_MS="30000"                                            # 可选

# Kimi (月之暗面)
export KIMI_API_KEY="..."
# 或者
export MOONSHOT_API_KEY="..."
export KIMI_BASE_URL="https://api.moonshot.cn/v1"  # 可选
export KIMI_TIMEOUT_MS="30000"                     # 可选
```

### 2. 代码配置

```rust
use llm_connector::{Config, ProviderConfig};

let config = Config {
    openai: Some(ProviderConfig {
        api_key: "sk-...".to_string(),
        base_url: Some("https://api.openai.com/v1".to_string()),
        timeout_ms: Some(30000),
    }),
    anthropic: Some(ProviderConfig {
        api_key: "sk-ant-...".to_string(),
        base_url: Some("https://api.anthropic.com".to_string()),
        timeout_ms: Some(30000),
    }),
    deepseek: Some(ProviderConfig {
        api_key: "sk-...".to_string(),
        base_url: Some("https://api.deepseek.com/v1".to_string()),
        timeout_ms: Some(30000),
    }),
    // ... 其他提供商
    ..Default::default()
};

let client = Client::with_config(config);
```

## 配置字段说明

### ProviderConfig

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `api_key` | String | ✅ | - | API 密钥 |
| `base_url` | Option\<String\> | ❌ | 提供商默认 URL | API 基础 URL |
| `timeout_ms` | Option\<u64\> | ❌ | 30000 | 请求超时时间（毫秒） |

### 默认 Base URL

| 提供商 | 默认 Base URL |
|--------|---------------|
| OpenAI | `https://api.openai.com/v1` |
| Anthropic | `https://api.anthropic.com` |
| DeepSeek | `https://api.deepseek.com/v1` |
| GLM | `https://open.bigmodel.cn/api/paas/v4` |
| Qwen | `https://dashscope.aliyuncs.com/compatible-mode/v1` |
| Kimi | `https://api.moonshot.cn/v1` |

## 环境变量命名规范

### 基本格式
```
{PROVIDER}_{FIELD}
```

### 别名支持
- GLM: `GLM_*` 或 `ZHIPU_*`
- Qwen: `QWEN_*` 或 `ALIBABA_QWEN_*`
- Kimi: `KIMI_*` 或 `MOONSHOT_*`

### 示例
```bash
# 标准命名
export OPENAI_API_KEY="sk-..."
export OPENAI_BASE_URL="https://api.openai.com/v1"
export OPENAI_TIMEOUT_MS="30000"

# 别名命名
export ZHIPU_API_KEY="..."  # 等同于 GLM_API_KEY
export MOONSHOT_API_KEY="..." # 等同于 KIMI_API_KEY
```

## 配置验证

### 必填字段验证
- 每个提供商必须有 `api_key`
- `api_key` 不能为空字符串

### 格式验证
- `timeout_ms` 必须大于 0
- `base_url` 必须是有效的 URL 格式

### 运行时验证
```rust
// 配置验证在客户端初始化时进行
let client = Client::from_env(); // 自动验证环境变量配置
let client = Client::with_config(config); // 验证传入的配置
```

## 最佳实践

### 1. 安全性
- ✅ 使用环境变量存储 API 密钥
- ✅ 不要将密钥写入配置文件
- ✅ 使用 `.env` 文件进行本地开发
- ❌ 不要将密钥提交到版本控制

### 2. 配置管理
- ✅ 优先使用环境变量
- ✅ 为不同环境使用不同的配置
- ✅ 设置合理的超时时间
- ❌ 避免硬编码配置

### 3. 开发环境
```bash
# .env 文件示例
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
DEEPSEEK_API_KEY=sk-...
```

### 4. 生产环境
- 使用 Kubernetes Secrets
- 使用 AWS Secrets Manager
- 使用 HashiCorp Vault
- 使用环境变量注入

## 错误处理

### 配置错误类型
```rust
pub enum LlmConnectorError {
    ConfigError(String),
    // ...
}
```

### 常见错误
- `ConfigError("Missing API key for provider: openai")`
- `ConfigError("Invalid timeout value: must be > 0")`
- `ConfigError("Invalid base URL format")`

## 迁移指南

### 从复杂配置迁移
如果您之前使用了包含路由、负载均衡等复杂配置，请：

1. **提取提供商配置**：只保留 API 密钥和端点配置
2. **移除路由配置**：使用外部负载均衡器
3. **简化环境变量**：使用标准命名格式
4. **更新代码**：使用新的 Client API

### 示例迁移
```rust
// 旧版本（复杂配置）
let config = ComplexGatewayConfig {
    routing: RoutingConfig { /* ... */ },
    circuit_breaker: CircuitBreakerConfig { /* ... */ },
    deployments: vec![/* ... */],
    // ...
};

// 新版本（简化配置）
let client = Client::from_env(); // 就这么简单！
```
