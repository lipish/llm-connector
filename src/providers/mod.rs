//! V2服务提供商模块
//!
//! 这个模块包含所有服务提供商的实现，每个提供商代表一个具体的LLM服务。

pub mod openai;
pub mod aliyun;
pub mod anthropic;
pub mod zhipu;
pub mod ollama;

// 重新导出服务提供商类型和函数
pub use openai::{
    OpenAIProvider,
    openai,
    openai_with_base_url,
    openai_with_config,
    azure_openai,
    openai_compatible,
};

pub use aliyun::{
    AliyunProvider,
    aliyun,
    aliyun_with_config,
    aliyun_international,
    aliyun_private,
    aliyun_with_timeout,
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
    zhipu,
    zhipu_openai_compatible,
    zhipu_with_config,
    zhipu_default,
    zhipu_with_timeout,
    zhipu_enterprise,
    validate_zhipu_key,
};

pub use ollama::{
    OllamaProvider,
    OllamaModelInfo,
    OllamaModelDetails,
    ollama,
    ollama_with_url,
    ollama_with_config,
};
