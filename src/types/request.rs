//! Request types for chat completions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message (instructions)
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool response message
    Tool,
}

/// Chat completion request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl ChatRequest {
    /// Create a new chat request with the given model
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Create a new chat request with model and initial messages
    pub fn new_with_messages(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            ..Default::default()
        }
    }

    /// Set the messages for the request
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    /// Add a single message to the request
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the top_p parameter
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set the maximum number of tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Enable streaming
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Set stop sequences
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Set presence penalty
    pub fn with_presence_penalty(mut self, penalty: f32) -> Self {
        self.presence_penalty = Some(penalty);
        self
    }

    /// Set frequency penalty
    pub fn with_frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = Some(penalty);
        self
    }

    /// Set the user identifier
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set the random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the tools
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set the tool choice
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Set the response format
    pub fn with_response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: Role,

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

    /// Provider-specific reasoning content (GLM 风格)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,

    /// Provider-specific reasoning (Qwen/DeepSeek/OpenAI o1 通用键)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,

    /// Provider-specific thought (OpenAI o1 键)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,

    /// Provider-specific thinking (Anthropic 键)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            role: Role::User,
            content: String::new(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
            reasoning: None,
            thought: None,
            thinking: None,
        }
    }
}

impl Message {
    /// Create a new message with the given role and content
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
            reasoning: None,
            thought: None,
            thinking: None,
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }

    /// Convenience: get the first available reasoning-like content
    pub fn reasoning_any(&self) -> Option<&str> {
        self.reasoning_content
            .as_deref()
            .or(self.reasoning.as_deref())
            .or(self.thought.as_deref())
            .or(self.thinking.as_deref())
    }

    /// Provider-agnostic post-processor: populate reasoning synonyms from raw JSON
    /// Scans nested JSON objects/arrays and fills each synonym field if present.
    pub fn populate_reasoning_from_json(&mut self, raw: &serde_json::Value) {
        fn collect_synonyms(val: &serde_json::Value, acc: &mut std::collections::HashMap<String, String>) {
            match val {
                serde_json::Value::Array(arr) => {
                    for v in arr { collect_synonyms(v, acc); }
                }
                serde_json::Value::Object(map) => {
                    for (k, v) in map {
                        let key = k.to_ascii_lowercase();
                        if let serde_json::Value::String(s) = v {
                            match key.as_str() {
                                "reasoning_content" | "reasoning" | "thought" | "thinking" => {
                                    acc.entry(key).or_insert_with(|| s.clone());
                                }
                                _ => {}
                            }
                        }
                        collect_synonyms(v, acc);
                    }
                }
                _ => {}
            }
        }

        let mut found = std::collections::HashMap::<String, String>::new();
        collect_synonyms(raw, &mut found);

        if self.reasoning_content.is_none() {
            if let Some(v) = found.get("reasoning_content") { self.reasoning_content = Some(v.clone()); }
        }
        if self.reasoning.is_none() {
            if let Some(v) = found.get("reasoning") { self.reasoning = Some(v.clone()); }
        }
        if self.thought.is_none() {
            if let Some(v) = found.get("thought") { self.thought = Some(v.clone()); }
        }
        if self.thinking.is_none() {
            if let Some(v) = found.get("thinking") { self.thinking = Some(v.clone()); }
        }
    }

    /// Create a tool response message
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
            name: None,
            tool_calls: None,
            ..Default::default()
        }
    }

    /// Set the name of the message sender
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set tool calls for assistant messages
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
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
    /// String mode: "none", "auto", or "required"
    Mode(String),
    /// Specific function to call
    Function {
        /// Type of tool (always "function")
        #[serde(rename = "type")]
        tool_type: String,
        /// Function to call
        function: FunctionChoice,
    },
}

impl ToolChoice {
    /// No tools should be called
    pub fn none() -> Self {
        Self::Mode("none".to_string())
    }

    /// Let the model decide
    pub fn auto() -> Self {
        Self::Mode("auto".to_string())
    }

    /// Tools must be called
    pub fn required() -> Self {
        Self::Mode("required".to_string())
    }

    /// Call a specific function
    pub fn function(name: impl Into<String>) -> Self {
        Self::Function {
            tool_type: "function".to_string(),
            function: FunctionChoice {
                name: name.into(),
            },
        }
    }
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
