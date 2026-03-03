//! Protocol Module - Public Standard Protocols
//!
//! This module only contains industry-recognized standard LLM API protocols:
//!
//! ## standardprotocol
//! - **OpenAI Protocol**: Standard OpenAI API specification - supported by multiple service providers
//! - **Anthropic Protocol**: Standard Anthropic Claude API specification - official protocol
//!
//! ## Design Principles
//! - Only contains public, standardized protocols
//! - Other service providers may implement these protocols
//! - Private protocols are defined in respective `providers` modules
//!
//! Note: Specific service provider implementations are in the `providers` module.

pub mod aliyun;
pub mod anthropic;
pub mod google;
pub mod ollama;
pub mod openai;
#[cfg(feature = "tencent")]
pub mod tencent;
pub mod utils;
pub mod zhipu;

// Re-export standard protocol types
pub use aliyun::AliyunProtocol;
pub use anthropic::AnthropicProtocol;
pub use google::GoogleProtocol;
pub use ollama::OllamaProtocol;
pub use openai::OpenAIProtocol;
#[cfg(feature = "tencent")]
pub use tencent::TencentNativeProtocol;
pub use zhipu::ZhipuProtocol;
