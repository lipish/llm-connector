//! Protocol Module - Public Standard Protocols
//!
//! Reorganized into V2 Architecture:
//! - common/: Shared components (Auth, Streamers, OpenAI-base)
//! - [provider]/: Implementation for each specific protocol

pub mod common;

pub mod aliyun;
pub mod anthropic;
pub mod google;
pub mod ollama;
pub mod openai;
#[cfg(feature = "tencent")]
pub mod tencent;
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
