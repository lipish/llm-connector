//! Protocol-based provider implementations
//! 
//! This module organizes providers by the API protocol they follow, rather than
//! by individual provider names. This approach recognizes that many providers
//! implement the same underlying protocol (especially OpenAI-compatible APIs).
//!
//! ## Supported Protocols
//!
//! ### OpenAI Protocol
//! The most widely adopted protocol, used by:
//! - DeepSeek, Zhipu, Moonshot, VolcEngine, Tencent, MiniMax, StepFun
//! - Standard `/chat/completions` endpoint
//! - Compatible request/response format
//!
//! ### Anthropic Protocol  
//! Used by Anthropic's Claude models:
//! - `/v1/messages` endpoint
//! - Different response structure with `content` array
//! - Different streaming format
//!
//! ### Aliyun Protocol
//! Custom protocol used by Aliyun DashScope:
//! - Nested `input` and `parameters` structure
//! - Different endpoint and response format

pub mod aliyun;
pub mod anthropic;
pub mod core;
pub mod factory;
pub mod openai;

// Re-export core components
pub use core::{Provider, ProviderAdapter, ErrorMapper, HttpTransport, GenericProvider};

// Re-export protocols
pub use openai::{OpenAIProtocol, openai_providers};
pub use anthropic::{AnthropicProtocol, anthropic_providers};
pub use aliyun::{AliyunProtocol, aliyun_providers};

// Re-export factory
pub use factory::{
    ProtocolFactory, ProtocolFactoryRegistry,
    OpenAIProtocolFactory, AnthropicProtocolFactory, AliyunProtocolFactory,
};

// Re-export configuration from the unified config module
pub use crate::config::{ProviderConfig, RetryConfig, RegistryConfig};
