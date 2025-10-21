//! Core types for llm-connector

mod request;
mod response;
mod message_block;

#[cfg(feature = "streaming")]
mod streaming;

// Re-exports
pub use request::*;
pub use response::*;
pub use message_block::*;

#[cfg(feature = "streaming")]
pub use streaming::*;

// Compatibility alias for users expecting ChatMessage
// ChatMessage is the same as Message
pub type ChatMessage = request::Message;
