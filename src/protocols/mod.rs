//! Protocol-based provider implementations
//!
//! This module organizes providers by the API protocol they follow, rather than
//! by individual provider names. This approach recognizes that many providers
//! implement the same underlying protocol (especially OpenAI-compatible APIs).
//!
//! # Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Application Layer                        │
//! │              (Client, ProviderRegistry)                     │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Provider Trait                           │
//! │         (Public API: chat(), chat_stream())                 │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              GenericProvider<Adapter>                       │
//! │    (Universal implementation for all protocols)             │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                ┌─────────────┼─────────────┐
//!                ▼             ▼             ▼
//!    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
//!    │   OpenAI     │  │  Anthropic   │  │   Aliyun     │
//!    │  Protocol    │  │  Protocol    │  │  Protocol    │
//!    └──────────────┘  └──────────────┘  └──────────────┘
//!           │                 │                 │
//!    ┌──────┴────┐           │                 │
//!    ▼           ▼           ▼                 ▼
//! DeepSeek   Zhipu      Claude            Qwen
//! Moonshot   Kimi
//! VolcEngine
//! Tencent
//! MiniMax
//! StepFun
//! LongCat
//! ```
//!
//! # Module Structure
//!
//! ## Core Module (`core.rs`)
//!
//! Defines the fundamental abstractions:
//! - **`Provider`** - Public trait for external API
//! - **`ProviderAdapter`** - Internal trait for protocol implementations
//! - **`GenericProvider<A>`** - Universal provider implementation
//! - **`HttpTransport`** - Shared HTTP client and configuration
//! - **`ErrorMapper`** - Protocol-specific error handling
//!
//! ## Protocol Modules
//!
//! ### OpenAI Protocol (`openai.rs`)
//!
//! The most widely adopted protocol, used by 8 providers:
//! - **DeepSeek** - `deepseek()` - DeepSeek-V3, DeepSeek-Chat
//! - **Zhipu (GLM)** - `zhipu()` - GLM-4, GLM-4-Plus, GLM-4-Flash
//! - **Moonshot (Kimi)** - `moonshot()` - Moonshot-v1 series
//! - **VolcEngine (Doubao)** - `volcengine()` - Doubao models
//! - **Tencent (Hunyuan)** - `tencent()` - Hunyuan models
//! - **MiniMax** - `minimax()` - MiniMax models
//! - **StepFun** - `stepfun()` - Step series models
//! - **LongCat** - `longcat()` - Free quota available for testing
//!
//! **Endpoint**: `POST /v1/chat/completions`
//!
//! ### Anthropic Protocol (`anthropic.rs`)
//!
//! Used by Anthropic's Claude models:
//! - **Claude** - `claude()` - Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku
//!
//! **Endpoint**: `POST /v1/messages`
//!
//! **Key Differences**:
//! - Separate `system` field instead of system message
//! - Required `max_tokens` field
//! - Response content is an array of blocks
//! - Different streaming event types
//!
//! ### Aliyun Protocol (`aliyun.rs`)
//!
//! Custom protocol used by Aliyun DashScope:
//! - **Qwen** - `qwen()` - Qwen-Max, Qwen-Plus, Qwen-Turbo
//!
//! **Endpoint**: `POST /services/aigc/text-generation/generation`
//!
//! **Key Differences**:
//! - Nested `input` and `parameters` structure
//! - Response data in `output.choices`
//! - Different field names for usage statistics
//!
//! ## Factory Module (`factory.rs`)
//!
//! Provides dynamic protocol creation for YAML configuration:
//! - **`ProtocolFactory`** - Trait for creating protocol adapters
//! - **`ProtocolFactoryRegistry`** - Manages all protocol factories
//! - **Built-in factories**: OpenAI, Anthropic, Aliyun
//!
//! # Usage Examples
//!
//! ## Direct Provider Creation
//!
//! ```rust
//! use llm_connector::{
//!     config::ProviderConfig,
//!     protocols::{core::GenericProvider, openai::deepseek},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ProviderConfig::new("your-api-key");
//! let provider = GenericProvider::new(config, deepseek())?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Provider Registry
//!
//! ```rust,no_run
//! use llm_connector::config::RegistryConfig;
//! use llm_connector::registry::ProviderRegistry;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RegistryConfig::from_yaml_file("config.yaml")?;
//! let registry = ProviderRegistry::from_config(config)?;
//! let provider = registry.get("deepseek").unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! # Adding New Providers
//!
//! ## For OpenAI-Compatible Providers (3 lines)
//!
//! ```rust
//! use llm_connector::protocols::openai::OpenAIProtocol;
//!
//! pub fn my_provider() -> OpenAIProtocol {
//!     OpenAIProtocol::new("my-provider", "https://api.example.com/v1", vec!["model-1"])
//! }
//! ```
//!
//! ## For Custom Protocols (~300 lines)
//!
//! Only needed if the provider uses a truly different protocol:
//! 1. Define request/response structures
//! 2. Implement `ProviderAdapter` trait
//! 3. Implement `ErrorMapper` trait
//! 4. Create factory (optional, for YAML support)

pub mod aliyun;
pub mod anthropic;
pub mod core;
pub mod factory;
pub mod openai;

// Re-export core components
pub use core::{ErrorMapper, GenericProvider, HttpTransport, Provider, ProviderAdapter};

// Re-export protocols
pub use aliyun::{aliyun_providers, AliyunProtocol};
pub use anthropic::{anthropic_providers, AnthropicProtocol};
pub use openai::{openai_providers, OpenAIProtocol};

// Re-export factory
pub use factory::{
    AliyunProtocolFactory, AnthropicProtocolFactory, OpenAIProtocolFactory, ProtocolFactory,
    ProtocolFactoryRegistry,
};

// Re-export configuration from the unified config module
pub use crate::config::{ProviderConfig, RegistryConfig, RetryConfig};
