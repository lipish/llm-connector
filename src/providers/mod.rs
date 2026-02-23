//! V2 Service Provider Module
//!
//! This module contains all service Provider implementations，each Provider represents a specific LLM service。

pub mod aliyun;
pub mod anthropic;
pub mod deepseek;
pub mod google;
pub mod longcat;
pub mod mock;
pub mod moonshot;
pub mod ollama;
pub mod openai;
pub mod tencent;
pub mod volcengine;
pub mod xiaomi;
pub mod zhipu;

// Re-export service Provider types and functions
pub use openai::{
    OpenAIProvider, azure_openai, openai, openai_compatible, openai_with_base_url,
    openai_with_config, validate_openai_key,
};

pub use aliyun::{
    AliyunProtocol, AliyunProvider, aliyun, aliyun_international, aliyun_private,
    aliyun_with_config, aliyun_with_timeout, validate_aliyun_key,
};

pub use anthropic::{
    AnthropicProvider, anthropic, anthropic_bedrock, anthropic_vertex, anthropic_with_config,
    anthropic_with_timeout, validate_anthropic_key,
};

pub use zhipu::{
    ZhipuProtocol, ZhipuProvider, validate_zhipu_key, zhipu, zhipu_enterprise,
    zhipu_openai_compatible, zhipu_with_config, zhipu_with_timeout,
};

pub use ollama::{
    OllamaModelDetails, OllamaModelInfo, OllamaProvider, ollama, ollama_with_base_url,
    ollama_with_config,
};

pub use longcat::{
    LongCatAnthropicProtocol, LongCatAnthropicProvider, longcat_anthropic,
    longcat_anthropic_with_config,
};

pub use volcengine::{VolcengineProtocol, VolcengineProvider, volcengine, volcengine_with_config};

#[cfg(feature = "tencent")]
pub use tencent::{TencentProvider, tencent, tencent_with_config};

pub use moonshot::{MoonshotProtocol, MoonshotProvider, moonshot, moonshot_with_config};

pub use deepseek::{DeepSeekProtocol, DeepSeekProvider, deepseek, deepseek_with_config};

pub use google::{GoogleProvider, google, google_with_config};

pub use xiaomi::{XiaomiProtocol, XiaomiProvider, xiaomi, xiaomi_with_config};
