//! V2协议模块
//!
//! 这个模块包含所有协议实现，每个协议代表一个标准的LLM API规范。

pub mod openai;
pub mod aliyun;
pub mod anthropic;
pub mod zhipu;

// 重新导出协议类型
pub use openai::OpenAIProtocol;
pub use aliyun::AliyunProtocol;
pub use anthropic::AnthropicProtocol;
pub use zhipu::ZhipuProtocol;
