//! Request types for chat completions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model identifier (e.g., "openai/gpt-4", "deepseek/deepseek-chat")
    pub model: String,
    
    /// List of messages in the conversation
    pub messages: Vec<Message>,
    
    /// Sampling temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Nucleus sampling parameter (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    
    /// Presence penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    
    /// Frequency penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    /// Logit bias
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f32>>,
    
    /// User identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    
    /// Random seed for deterministic outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    
    /// Tools available to the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    
    /// Tool choice strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    
    /// Response format specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            model: String::new(),
            messages: Vec::new(),
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            seed: None,
            tools: None,
            tool_choice: None,
            response_format: None,
        }
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: String,
    
    /// Content of the message
    pub content: String,
    
    /// Name of the message sender (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// Tool calls made by the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    
    /// Tool call ID (for tool responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            role: String::new(),
            content: String::new(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Type of tool (usually "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    
    /// Function definition
    pub function: Function,
}

/// Function definition for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// Function name
    pub name: String,
    
    /// Function description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Function parameters schema
    pub parameters: serde_json::Value,
}

/// Tool choice strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tools should be called
    None,
    /// Let the model decide
    Auto,
    /// Tools must be called
    Required,
    /// Specific function to call
    Function { function: FunctionChoice },
}

/// Specific function choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    /// Name of the function to call
    pub name: String,
}

/// Tool call made by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for the tool call
    pub id: String,
    
    /// Type of tool call (usually "function")
    #[serde(rename = "type")]
    pub call_type: String,
    
    /// Function call details
    pub function: FunctionCall,
}

/// Function call details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the function
    pub name: String,
    
    /// Arguments as JSON string
    pub arguments: String,
}

/// Response format specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// Type of response format ("text" or "json_object")
    #[serde(rename = "type")]
    pub format_type: String,
}
