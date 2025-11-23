//! V2架构核心模块
//!
//! 这个模块ContainsV2架构的所有核心组件：
//! - 统一的traitDefine (Protocol, Provider)
//! - HTTP客户端实现
//! - 通用Provide商实现

pub mod traits;
pub mod client;
pub mod builder;
pub mod configurable;

// 重新导出核心类型
pub use traits::{Protocol, Provider, GenericProvider};
pub use client::HttpClient;
pub use builder::ProviderBuilder;
pub use configurable::{ConfigurableProtocol, ProtocolConfig, EndpointConfig, AuthConfig};
