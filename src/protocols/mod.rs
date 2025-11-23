//! protocol模块 - 公开标准protocol
//!
//! this模块只Contains业界公认标准LLM APIprotocol：
//!
//! ## 标准protocol
//! - **OpenAI Protocol**: 标准OpenAI API规范 - 被多个服务Provide商Support
//! - **Anthropic Protocol**: 标准Anthropic Claude API规范 - 官方protocol
//!
//! ## 设计原则
//! - 只Contains公开、标准化protocol
//! - 其他服务Provide商maywill实现theseprotocol
//! - 私有protocolDefinein各自 `providers` 模块中
//!
//! Note：具体服务Provide商实现in `providers` 模块中。

pub mod openai;
pub mod anthropic;

// 重新导出标准protocol类型
pub use openai::OpenAIProtocol;
pub use anthropic::AnthropicProtocol;
