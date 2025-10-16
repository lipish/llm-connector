//! V1架构 - 传统架构实现
//!
//! 这个模块包含了llm-connector的V1架构实现，保留用于向后兼容。
//! 新项目建议使用V2架构。

pub mod client;
pub mod core;
pub mod protocols;
pub mod providers;
pub mod ollama;

// 重新导出主要类型以保持兼容性
pub use client::LlmClient;
pub use core::{GenericProvider, HttpClient, ProviderAdapter};

// 重新导出协议
pub use protocols::{
    OpenAIProtocol,
    AnthropicProtocol,
};

// 重新导出提供商
pub use providers::{
    AliyunProvider,
    OllamaProvider,
    TencentProvider,
    ZhipuProvider,
};

// 重新导出Ollama特殊功能
pub use ollama::{
    OllamaModel,
    OllamaModelInfo,
    OllamaModelDetails,
    OllamaCreateRequest,
    OllamaDeleteRequest,
    OllamaPullRequest,
    OllamaShowRequest,
    OllamaListRequest,
    OllamaTagsResponse,
    OllamaShowResponse,
    OllamaPullResponse,
    OllamaCreateResponse,
    OllamaDeleteResponse,
};
