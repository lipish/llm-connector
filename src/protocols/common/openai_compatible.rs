use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, ReasoningEffort, ToolCall};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentBlockMode {
    Standard,
    TextOnly,
}

#[derive(Clone, Debug)]
pub struct OpenAICompatibleCapabilities {
    pub content_block_mode: ContentBlockMode,
    pub supports_response_format: bool,
    pub supports_reasoning_effort: bool,
}

impl Default for OpenAICompatibleCapabilities {
    fn default() -> Self {
        Self {
            content_block_mode: ContentBlockMode::Standard,
            supports_response_format: true,
            supports_reasoning_effort: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OpenAICompatibleRequestParts {
    pub messages: Vec<serde_json::Value>,
    pub tools: Option<Vec<serde_json::Value>>,
    pub tool_choice: Option<serde_json::Value>,
    pub response_format: Option<serde_json::Value>,
    pub reasoning_effort: Option<ReasoningEffort>,
}

#[derive(Clone, Debug)]
pub struct NormalizedContent {
    pub content: String,
    pub reasoning: Option<String>,
}

pub fn map_openai_compatible_messages(
    request: &ChatRequest,
    capabilities: &OpenAICompatibleCapabilities,
) -> Result<Vec<serde_json::Value>, LlmConnectorError> {
    match capabilities.content_block_mode {
        ContentBlockMode::Standard => {
            Ok(crate::protocols::common::request::openai_message_converter(&request.messages))
        }
        ContentBlockMode::TextOnly => {
            crate::protocols::common::request::openai_message_converter_downgrade(&request.messages)
        }
    }
}

pub fn map_openai_compatible_tools(
    request: &ChatRequest,
) -> Option<Vec<serde_json::Value>> {
    request.tools.as_ref().map(|tools| {
        tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "type": tool.tool_type,
                    "function": {
                        "name": tool.function.name,
                        "description": tool.function.description,
                        "parameters": tool.function.parameters,
                    }
                })
            })
            .collect()
    })
}

pub fn map_openai_compatible_tool_choice(request: &ChatRequest) -> Option<serde_json::Value> {
    request
        .tool_choice
        .as_ref()
        .map(|choice| serde_json::to_value(choice).unwrap_or(serde_json::json!("auto")))
}

pub fn map_openai_compatible_response_format(
    request: &ChatRequest,
    capabilities: &OpenAICompatibleCapabilities,
) -> Option<serde_json::Value> {
    if capabilities.supports_response_format {
        request
            .response_format
            .as_ref()
            .map(|rf| serde_json::to_value(rf).unwrap_or(serde_json::json!({"type": "text"})))
    } else {
        None
    }
}

pub fn map_openai_compatible_reasoning_effort(
    request: &ChatRequest,
    capabilities: &OpenAICompatibleCapabilities,
) -> Option<ReasoningEffort> {
    if capabilities.supports_reasoning_effort {
        request.reasoning_effort
    } else {
        None
    }
}

pub fn normalize_openai_compatible_content(
    content: Option<String>,
    reasoning_content: Option<String>,
) -> NormalizedContent {
    let mut content_str = content.unwrap_or_default();
    let mut reasoning_str = reasoning_content;

    if reasoning_str.is_none()
        && content_str.contains("<think>")
        && let Some(start_idx) = content_str.find("<think>")
        && let Some(end_idx) = content_str.find("</think>")
    {
        let extracted_reasoning = content_str[start_idx + 7..end_idx].to_string();
        reasoning_str = Some(extracted_reasoning);

        let mut new_content = content_str[..start_idx].to_string();
        new_content.push_str(&content_str[end_idx + 8..]);
        content_str = new_content.trim().to_string();
    }

    NormalizedContent {
        content: content_str,
        reasoning: reasoning_str,
    }
}

pub fn map_openai_compatible_tool_calls(
    tool_calls: Option<serde_json::Value>,
) -> Option<Vec<ToolCall>> {
    tool_calls.and_then(|tc_val| serde_json::from_value::<Vec<ToolCall>>(tc_val).ok())
}

pub fn build_openai_compatible_request_parts(
    request: &ChatRequest,
    capabilities: &OpenAICompatibleCapabilities,
) -> Result<OpenAICompatibleRequestParts, LlmConnectorError> {
    let messages = map_openai_compatible_messages(request, capabilities)?;
    let tools = map_openai_compatible_tools(request);
    let tool_choice = map_openai_compatible_tool_choice(request);
    let response_format = map_openai_compatible_response_format(request, capabilities);
    let reasoning_effort = map_openai_compatible_reasoning_effort(request, capabilities);

    Ok(OpenAICompatibleRequestParts {
        messages,
        tools,
        tool_choice,
        response_format,
        reasoning_effort,
    })
}

pub fn parse_openai_compatible_chat_response(
    response: &str,
    provider_name: &str,
) -> Result<ChatResponse, LlmConnectorError> {
    crate::protocols::formats::chat_completions::parse_chat_completions_chat_response(
        response,
        provider_name,
    )
}

#[cfg(feature = "streaming")]
pub fn parse_openai_compatible_stream(
    response: reqwest::Response,
    mode: crate::sse::StreamingParseMode,
) -> crate::types::ChatStream {
    crate::sse::sse_to_streaming_response_with_mode(response, mode)
}
