//! Core types for llm-connector

mod message_block;
mod request;
mod response;

#[cfg(feature = "streaming")]
mod streaming;

// Re-exports
pub use message_block::*;
pub use request::*;
pub use response::*;

#[cfg(feature = "streaming")]
pub use streaming::*;

// Compatibility alias for users expecting ChatMessage
// ChatMessage is the same as Message
pub type ChatMessage = request::Message;
