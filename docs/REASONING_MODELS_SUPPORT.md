# Universal Reasoning Models Support

## Overview

llm-connector now provides unified streaming response support for various reasoning models. No matter which field contains the reasoning content (`reasoning_content`, `reasoning`, `thought`, `thinking`), it's automatically detected and extracted.

## Quick Comparison Table

| Provider | Model Example | Reasoning Field | Status | Priority |
|----------|--------------|----------------|--------|----------|
| **Volcengine** | Doubao-Seed-Code | `reasoning_content` | Verified | 2 |
| **DeepSeek** | DeepSeek R1 | `reasoning_content` / `reasoning` | Supported | 2/3 |
| **OpenAI** | o1-preview, o1-mini | `thought` / `reasoning_content` | Supported | 4/2 |
| **Qwen** | Qwen-Plus | `reasoning` | Supported | 3 |
| **Anthropic** | Claude 3.5 Sonnet | `thinking` | Supported | 5 |
| **Standard Models** | GPT-4, Claude, etc. | `content` | Unaffected | 1 |

**Note**: Lower priority number = higher priority. When multiple fields exist, the highest priority field is used.

## Supported Reasoning Models

### 1. Volcengine Doubao-Seed-Code

**Field**: `delta.reasoning_content`

**Response Format**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "reasoning_content": "I need to introduce myself in one sentence..."
    }
  }]
}
```

**Usage Example**:
```rust
use llm_connector::LlmClient;
use llm_connector::types::{ChatRequest, Message, Role};
use futures_util::StreamExt;

let client = LlmClient::volcengine("api-key")?;
let request = ChatRequest {
    model: "ep-20250118155555-xxxxx".to_string(),
    messages: vec![Message::text(Role::User, "Introduce yourself")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // Automatically extracts reasoning_content
    }
}
```

### 2. DeepSeek R1

**Fields**: `delta.reasoning_content` or `delta.reasoning`

**Response Format**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "reasoning_content": "Let me think about this step by step..."
    }
  }]
}
```

**Usage Example**:
```rust
let client = LlmClient::deepseek("api-key")?;
let request = ChatRequest {
    model: "deepseek-reasoner".to_string(),
    messages: vec![Message::text(Role::User, "Solve this problem")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // Automatically extracts reasoning content
    }
}
```

### 3. OpenAI o1 Series

**Fields**: `delta.thought` or `delta.reasoning_content`

**Response Format**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "thought": "Analyzing the problem..."
    }
  }]
}
```

**Usage Example**:
```rust
let client = LlmClient::openai("api-key")?;
let request = ChatRequest {
    model: "o1-preview".to_string(),
    messages: vec![Message::text(Role::User, "Explain quantum computing")],
    stream: Some(true),
    ..Default::default()
};

let mut stream = client.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // Automatically extracts thought content
    }
}
```

### 4. Qwen-Plus

**Field**: `delta.reasoning`

**Response Format**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "reasoning": "First, I need to understand..."
    }
  }]
}
```

### 5. Anthropic Claude with Thinking

**Field**: `delta.thinking`

**Response Format**:
```json
{
  "choices": [{
    "delta": {
      "content": "",
      "thinking": "Let me consider this carefully..."
    }
  }]
}
```

## How It Works

### Automatic Field Detection

The library uses a priority-based system to extract content:

1. **Priority 1**: `delta.content` (standard content field)
2. **Priority 2**: `delta.reasoning_content` (Volcengine, DeepSeek R1)
3. **Priority 3**: `delta.reasoning` (Qwen, DeepSeek)
4. **Priority 4**: `delta.thought` (OpenAI o1)
5. **Priority 5**: `delta.thinking` (Anthropic)

When you call `chunk.get_content()`, the library automatically checks these fields in order and returns the first non-empty one.

### Implementation

The extraction logic is in `src/types/streaming.rs`:

```rust
impl StreamingResponse {
    pub fn get_content(&self) -> Option<&str> {
        self.choices.first().and_then(|choice| {
            choice.delta.content.as_ref()
                .filter(|s| !s.is_empty())
                .or_else(|| choice.delta.reasoning_content.as_ref())
                .or_else(|| choice.delta.reasoning.as_ref())
                .or_else(|| choice.delta.thought.as_ref())
                .or_else(|| choice.delta.thinking.as_ref())
                .map(|s| s.as_str())
        })
    }
}
```

## Benefits

1. **Zero Configuration**: No need to specify which field to use
2. **Unified Interface**: Same code works for all reasoning models
3. **Backward Compatible**: Standard models (GPT-4, Claude) work as before
4. **Future-Proof**: Easy to add support for new reasoning fields

## Testing

All reasoning models have been tested with streaming responses. See `examples/volcengine_streaming.rs` for a complete example.

## Migration Guide

If you're using reasoning models, no changes are needed! The library automatically handles the extraction:

**Before** (manual extraction):
```rust
if let Some(reasoning) = chunk.choices.first()
    .and_then(|c| c.delta.reasoning_content.as_ref()) {
    print!("{}", reasoning);
}
```

**After** (automatic extraction):
```rust
if let Some(content) = chunk.get_content() {
    print!("{}", content);  // Works for all models
}
```

## Troubleshooting

### Content Not Appearing

If you're not seeing reasoning content:

1. Verify the model supports reasoning (check provider documentation)
2. Enable streaming: `stream: Some(true)`
3. Check the raw response to see which field contains the content
4. Report the field name so we can add support

### Multiple Fields Present

If a response contains multiple reasoning fields, the library uses the highest priority field. This ensures consistent behavior across providers.

## Future Enhancements

- Support for additional reasoning fields as new models are released
- Configurable field priority for advanced use cases
- Separate access to reasoning vs. final answer content

