//! V2 Service Provider Module
//!
//! This module contains all service Provider implementations，each Provider represents a specific LLM service。

pub mod openai;
pub mod aliyun;
pub mod anthropic;
pub mod zhipu;
pub mod ollama;
pub mod longcat;
pub mod volcengine;
pub mod tencent;
pub mod moonshot;
pub mod deepseek;
pub mod google;

// Re-export service Provider types and functions
pub use openai::{
    OpenAIProvider,
    openai,
    openai_with_base_url,
    openai_with_config,
    azure_openai,
    openai_compatible,
    validate_openai_key,
};

pub use aliyun::{
    AliyunProvider,
    AliyunProtocol,
    aliyun,
    aliyun_with_config,
    aliyun_international,
    aliyun_private,
    aliyun_with_timeout,
    validate_aliyun_key,
};

pub use anthropic::{
    AnthropicProvider,
    anthropic,
    anthropic_with_config,
    anthropic_vertex,
    anthropic_bedrock,
    anthropic_with_timeout,
    validate_anthropic_key,
};

pub use zhipu::{
    ZhipuProvider,
    ZhipuProtocol,
    zhipu,
    zhipu_openai_compatible,
    zhipu_with_config,
    zhipu_with_timeout,
    zhipu_enterprise,
    validate_zhipu_key,
};

pub use ollama::{
    OllamaProvider,
    OllamaModelInfo,
    OllamaModelDetails,
    ollama,
    ollama_with_base_url,
    ollama_with_config,
};

pub use longcat::{
    LongCatAnthropicProvider,
    LongCatAnthropicProtocol,
    longcat_anthropic,
    longcat_anthropic_with_config,
};

pub use volcengine::{
    VolcengineProvider,
    VolcengineProtocol,
    volcengine,
    volcengine_with_config,
};

pub use tencent::{
    TencentProvider,
    TencentProtocol,
    tencent,
    tencent_with_config,
};

pub use moonshot::{
    MoonshotProvider,
    MoonshotProtocol,
    moonshot,
    moonshot_with_config,
};

pub use deepseek::{
    DeepSeekProvider,
    DeepSeekProtocol,
    deepseek,
    deepseek_with_config,
};

pub use google::{
    GoogleProvider,
    google,
    google_with_config,
};
