# API Specification

llm-connector API 规范，定义统一的接口和数据格式。

## 设计原则

- **OpenAI 兼容**：遵循 OpenAI API 标准
- **提供商透明**：统一接口屏蔽提供商差异
- **类型安全**：强类型定义，编译时检查
- **扩展性**：易于添加新提供商和功能

## 核心 API

### Client

主要客户端接口，提供统一的 LLM 访问能力。

```rust
pub struct Client {
    // 内部实现
}

impl Client {
    /// 从环境变量创建客户端
    pub fn from_env() -> Self;
    
    /// 使用配置创建客户端
    pub fn with_config(config: Config) -> Self;
    
    /// 发送聊天完成请求
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmConnectorError>;
    
    /// 发送流式聊天完成请求
    #[cfg(feature = "streaming")]
    pub async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, LlmConnectorError>;
    
    /// 列出所有支持的模型
    pub fn list_models(&self) -> Vec<String>;
    
    /// 列出所有配置的提供商
    pub fn list_providers(&self) -> Vec<String>;
    
    /// 检查模型是否支持
    pub fn supports_model(&self, model: &str) -> bool;
    
    /// 获取模型的提供商信息
    pub fn get_provider_info(&self, model: &str) -> Option<String>;
}
```

### 请求类型

#### ChatRequest

聊天完成请求，兼容 OpenAI API 格式。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// 模型标识符 (e.g., "openai/gpt-4", "deepseek/deepseek-chat")
    pub model: String,
    
    /// 对话消息列表
    pub messages: Vec<Message>,
    
    /// 采样温度 (0.0 到 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// 核采样参数 (0.0 到 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// 最大生成 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// 是否流式返回
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    
    /// 停止序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    
    /// 存在惩罚 (-2.0 到 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    
    /// 频率惩罚 (-2.0 到 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    /// 用户标识符
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    
    /// 随机种子
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    
    /// 工具定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    
    /// 工具选择策略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    
    /// 响应格式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}
```

#### Message

对话消息。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息角色 ("system", "user", "assistant", "tool")
    pub role: String,
    
    /// 消息内容
    pub content: String,
    
    /// 发送者名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// 工具调用（助手消息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    
    /// 工具调用 ID（工具响应消息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}
```

### 响应类型

#### ChatResponse

聊天完成响应。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// 完成的唯一标识符
    pub id: String,
    
    /// 对象类型 (总是 "chat.completion")
    pub object: String,
    
    /// 创建时间戳
    pub created: u64,
    
    /// 使用的模型
    pub model: String,
    
    /// 完成选择列表
    pub choices: Vec<Choice>,
    
    /// 使用统计
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    
    /// 系统指纹
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}
```

#### Choice

完成选择。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// 选择索引
    pub index: u32,
    
    /// 生成的消息
    pub message: Message,
    
    /// 结束原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}
```

#### Usage

Token 使用统计。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// 提示 token 数
    pub prompt_tokens: u32,
    
    /// 完成 token 数
    pub completion_tokens: u32,
    
    /// 总 token 数
    pub total_tokens: u32,
}
```

### 流式类型

#### StreamingResponse

流式响应块。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResponse {
    /// 完成的唯一标识符
    pub id: String,
    
    /// 对象类型 (总是 "chat.completion.chunk")
    pub object: String,
    
    /// 创建时间戳
    pub created: u64,
    
    /// 使用的模型
    pub model: String,
    
    /// 流式选择列表
    pub choices: Vec<StreamingChoice>,
    
    /// 使用统计（仅在最后一块）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}
```

#### StreamingChoice

流式选择。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChoice {
    /// 选择索引
    pub index: u32,
    
    /// 增量内容
    pub delta: Delta,
    
    /// 结束原因（仅在最后一块）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}
```

#### Delta

增量内容。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    /// 消息角色（仅在第一块）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    
    /// 增量内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    
    /// 工具调用（函数调用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}
```

## 模型命名规范

### 格式

```
[provider/]model_name
```

### 示例

```rust
// 显式提供商前缀
"openai/gpt-4"
"anthropic/claude-3-haiku"
"deepseek/deepseek-chat"
"glm/glm-4"
"qwen/qwen-turbo"
"kimi/moonshot-v1-8k"

// 自动检测（基于模型名称模式）
"gpt-4"           // 自动识别为 OpenAI
"claude-3-haiku"  // 自动识别为 Anthropic
"deepseek-chat"   // 自动识别为 DeepSeek
"glm-4"           // 自动识别为 GLM
"qwen-turbo"      // 自动识别为 Qwen
"moonshot-v1-8k"  // 自动识别为 Kimi
```

### 提供商别名

| 标准名称 | 别名 |
|----------|------|
| glm | zhipu |
| qwen | alibaba |
| kimi | moonshot |

## 使用示例

### 基本聊天

```rust
use llm_connector::{Client, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env();
    
    let request = ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
                ..Default::default()
            }
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };
    
    let response = client.chat(request).await?;
    println!("Response: {}", response.choices[0].message.content);
    
    Ok(())
}
```

### 流式聊天

```rust
use llm_connector::{Client, ChatRequest, Message};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env();
    
    let request = ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Tell me a story".to_string(),
                ..Default::default()
            }
        ],
        stream: Some(true),
        ..Default::default()
    };
    
    let mut stream = client.chat_stream(request).await?;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(content) = chunk.choices[0].delta.content.as_ref() {
            print!("{}", content);
        }
    }
    
    Ok(())
}
```

### 多提供商支持

```rust
use llm_connector::{Client, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env();
    
    // 列出所有支持的模型
    let models = client.list_models();
    println!("Supported models: {:?}", models);
    
    // 检查特定模型是否支持
    if client.supports_model("gpt-4") {
        println!("GPT-4 is supported");
    }
    
    // 获取模型的提供商信息
    if let Some(provider) = client.get_provider_info("claude-3-haiku") {
        println!("Claude-3-Haiku provider: {}", provider);
    }
    
    Ok(())
}
```

## 错误处理

所有 API 调用都返回 `Result<T, LlmConnectorError>`，详见 [错误处理规范](./ERRORS.md)。

```rust
use llm_connector::{Client, LlmConnectorError};

match client.chat(request).await {
    Ok(response) => {
        // 处理成功响应
    },
    Err(LlmConnectorError::RateLimitError(msg)) => {
        // 处理速率限制
    },
    Err(LlmConnectorError::AuthenticationError(msg)) => {
        // 处理认证错误
    },
    Err(e) => {
        // 处理其他错误
    }
}
```
