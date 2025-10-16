//! V2架构核心模块
//!
//! 这个模块包含V2架构的所有核心组件：
//! - 统一的trait定义 (Protocol, Provider)
//! - HTTP客户端实现
//! - 通用提供商实现

pub mod traits;
pub mod client;

// 重新导出核心类型
pub use traits::{Protocol, Provider, GenericProvider};
pub use client::HttpClient;
