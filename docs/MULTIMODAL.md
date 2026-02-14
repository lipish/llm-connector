# Multi-modal Content Guide

llm-connector provides native support for multi-modal content (text + images) since v0.5.0.

## Basic Usage

Send text and images in a single message:

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, MessageBlock, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::openai("sk-...")?;

    // Text + Image URL
    let request = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            Message::new(
                Role::User,
                vec![
                    MessageBlock::text("What's in this image?"),
                    MessageBlock::image_url("https://example.com/image.jpg"),
                ],
            ),
        ],
        ..Default::default()
    };

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}
```

## Image from Base64

```rust
let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAUA..."; // Your base64 image data

let request = ChatRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        Message::new(
            Role::User,
            vec![
                MessageBlock::text("Analyze this image"),
                MessageBlock::image_base64("image/jpeg", base64_data),
            ],
        ),
    ],
    ..Default::default()
};
```

## Supported Content Types

| Method | Description |
|--------|-------------|
| `MessageBlock::text(text)` | Text content |
| `MessageBlock::image_url(url)` | Image from URL (OpenAI format) |
| `MessageBlock::image_base64(media_type, data)` | Base64 encoded image |
| `MessageBlock::image_url_anthropic(url)` | Image from URL (Anthropic format) |

## Provider Support

| Provider | Text | Images | Notes |
|----------|------|--------|-------|
| OpenAI | ✅ | ✅ | Full support (gpt-4o, gpt-4-vision) |
| Anthropic | ✅ | ✅ | Full support (Claude 3+) |
| Google | ✅ | ✅ | Full support (Gemini) |
| Aliyun | ✅ | ✅ | Qwen-VL models |
| Zhipu | ✅ | ✅ | GLM-4V models |
| Other | ✅ | ❌ | Text only (images converted to description) |

## Multiple Images

You can include multiple images in a single message:

```rust
let request = ChatRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        Message::new(
            Role::User,
            vec![
                MessageBlock::text("Compare these two images"),
                MessageBlock::image_url("https://example.com/image1.jpg"),
                MessageBlock::image_url("https://example.com/image2.jpg"),
            ],
        ),
    ],
    ..Default::default()
};
```

## Image with Detail Level (OpenAI)

```rust
// For OpenAI, you can specify detail level
let block = MessageBlock::ImageUrl {
    url: "https://example.com/image.jpg".to_string(),
    detail: Some("high".to_string()), // "low", "high", or "auto"
};
```

## Examples

```bash
cargo run --example multimodal_basic
```

See `examples/multimodal_basic.rs` for more examples.

