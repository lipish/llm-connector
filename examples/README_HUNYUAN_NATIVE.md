# 腾讯混元原生API使用指南

本文档介绍如何使用llm-connector库的腾讯混元原生API功能。

## 🚀 快速开始

### 1. 启用功能

在`Cargo.toml`中启用`tencent-native`功能：

```toml
[dependencies]
llm-connector = { version = "0.3.10", features = ["tencent-native"] }
tokio = { version = "1", features = ["full"] }

# 如果需要流式响应，同时启用streaming功能
llm-connector = { version = "0.3.10", features = ["tencent-native", "streaming"] }
```

### 2. 获取腾讯云凭证

1. 登录[腾讯云控制台](https://console.cloud.tencent.com/)
2. 访问[API密钥管理](https://console.cloud.tencent.com/cam/capi)
3. 创建或获取您的`SecretId`和`SecretKey`

### 3. 基础使用

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建腾讯混元原生API客户端
    let client = LlmClient::hunyuan_native(
        "your-secret-id",
        "your-secret-key", 
        Some("ap-beijing")  // 可选：指定地域
    );

    let request = ChatRequest {
        model: "hunyuan-lite".to_string(),
        messages: vec![Message::user("你好！")],
        ..Default::default()
    };

    let response = client.chat(&request).await?;
    println!("回复: {}", response.choices[0].message.content);
    Ok(())
}
```

## 🔧 配置选项

### 地域选择

腾讯混元支持多个地域，常用地域包括：

- `ap-beijing` - 北京（默认）
- `ap-shanghai` - 上海
- `ap-guangzhou` - 广州

```rust
// 指定地域
let client = LlmClient::hunyuan_native("secret-id", "secret-key", Some("ap-shanghai"));

// 使用默认地域（ap-beijing）
let client = LlmClient::hunyuan_native("secret-id", "secret-key", None);
```

### 自定义超时

```rust
let client = LlmClient::hunyuan_native_with_timeout(
    "secret-id", 
    "secret-key", 
    Some("ap-beijing"), 
    60000  // 60秒超时
);
```

## 🌊 流式响应

```rust
use futures_util::StreamExt;

let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if !chunk.content.is_empty() {
        print!("{}", chunk.content);
    }
}
```

## 🎯 支持的模型

- `hunyuan-lite` - 轻量版本，速度快
- `hunyuan-standard` - 标准版本，平衡性能和质量
- `hunyuan-pro` - 专业版本，最高质量

## 🔐 认证机制

腾讯混元原生API使用腾讯云的TC3-HMAC-SHA256签名认证：

1. **签名算法**: TC3-HMAC-SHA256
2. **认证头**: Authorization, X-TC-Action, X-TC-Version等
3. **时间戳**: 自动生成，防重放攻击
4. **地域**: 支持多地域部署

## 📝 环境变量

为了安全起见，建议使用环境变量存储凭证：

```bash
export TENCENT_SECRET_ID="your-secret-id"
export TENCENT_SECRET_KEY="your-secret-key"
export TENCENT_REGION="ap-beijing"  # 可选
export HUNYUAN_MODEL="hunyuan-lite"  # 可选
```

然后在代码中读取：

```rust
let secret_id = std::env::var("TENCENT_SECRET_ID")?;
let secret_key = std::env::var("TENCENT_SECRET_KEY")?;
let region = std::env::var("TENCENT_REGION").ok();

let client = LlmClient::hunyuan_native(&secret_id, &secret_key, region.as_deref());
```

## 🧪 运行示例

```bash
# 设置环境变量
export TENCENT_SECRET_ID="your-secret-id"
export TENCENT_SECRET_KEY="your-secret-key"

# 运行基础示例
cargo run --example hunyuan_native_basic --features tencent-native

# 运行流式示例
cargo run --example hunyuan_native_streaming --features "tencent-native,streaming"
```

## ⚡ 性能优势

相比OpenAI兼容接口，原生API具有以下优势：

1. **更好的错误处理**: 腾讯云原生错误码和消息
2. **完整功能支持**: 访问所有腾讯云特有功能
3. **更好的调试**: 详细的请求ID和错误信息
4. **地域支持**: 可以选择最近的服务器
5. **官方支持**: 使用腾讯云官方API规范

## 🔍 故障排除

### 常见错误

1. **认证失败**: 检查SecretId和SecretKey是否正确
2. **权限不足**: 确保账户有混元大模型访问权限
3. **地域错误**: 确认指定的地域支持混元服务
4. **网络问题**: 检查网络连接和防火墙设置

### 调试技巧

启用调试日志：

```bash
export LLM_DEBUG_REQUEST_RAW=1
export LLM_DEBUG_RESPONSE_RAW=1
```

这将输出详细的HTTP请求和响应信息，帮助诊断问题。

## 📚 更多资源

- [腾讯混元大模型官方文档](https://cloud.tencent.com/document/product/1729)
- [腾讯云API文档](https://cloud.tencent.com/document/api)
- [llm-connector项目主页](https://github.com/lipish/llm-connector)
