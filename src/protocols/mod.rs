//! 协议模块 - 公开标准协议
//!
//! 这个模块只Contains业界公认的标准LLM API协议：
//!
//! ## 标准协议
//! - **OpenAI Protocol**: 标准OpenAI API规范 - 被多个服务Provide商Support
//! - **Anthropic Protocol**: 标准Anthropic Claude API规范 - 官方协议
//!
//! ## 设计原则
//! - 只Contains公开、标准化的协议
//! - 其他服务Provide商可能会实现这些协议
//! - 私有协议Define在各自的 `providers` 模块中
//!
//! 注意：具体的服务Provide商实现在 `providers` 模块中。

pub mod openai;
pub mod anthropic;

// 重新导出标准协议类型
pub use openai::OpenAIProtocol;
pub use anthropic::AnthropicProtocol;
