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

pub mod anthropic;
pub mod openai;
pub mod tencent_native;

// Re-export standard protocol types
pub use anthropic::AnthropicProtocol;
pub use openai::OpenAIProtocol;
