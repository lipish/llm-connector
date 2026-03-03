//! Official LLM Endpoint Constants
//!
//! These constants are provided for reference and convenience.
//! The `llm-connector` library does not use these by default;
//! users must explicitly pass the desired `base_url` to the client.

/// OpenAI Official API V1
pub const OPENAI_API_V1: &str = "https://api.openai.com/v1";

/// Anthropic Official API V1
pub const ANTHROPIC_API_V1: &str = "https://api.anthropic.com";

/// Google Gemini API V1 Beta
pub const GOOGLE_GEMINI_V1BETA: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Zhipu BigModel API V4 (China)
pub const ZHIPU_CN_V4: &str = "https://open.bigmodel.cn/api/paas/v4";

/// Aliyun DashScope API V1
pub const ALIYUN_DASHSCOPE_V1: &str = "https://dashscope.aliyuncs.com";

/// Ollama Local Default
pub const OLLAMA_LOCAL: &str = "http://localhost:11434";

/// DeepSeek Official API
pub const DEEPSEEK_API: &str = "https://api.deepseek.com";

/// Moonshot Official API
pub const MOONSHOT_API: &str = "https://api.moonshot.cn";

/// Volcengine (Bytedance) Ark API
pub const VOLCENGINE_ARK: &str = "https://ark.cn-beijing.volces.com";
