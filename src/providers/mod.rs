//! V2服务提供商模块
//!
//! 这个模块包含所有服务提供商的实现，每个提供商代表一个具体的LLM服务。

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

// 重新导出服务提供商类型和函数
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
