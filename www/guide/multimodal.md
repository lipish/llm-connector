# Multi-modal

llm-connector provides native support for multi-modal content (text + images).

## Basic Usage

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, MessageBlock, Role}};

let client = LlmClient::openai("sk-...")?;

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
```

## Image from Base64

```rust
let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAUA...";

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

## Provider Support

| Provider | Text | Images |
|----------|------|--------|
| OpenAI | ✅ | ✅ |
| Anthropic | ✅ | ✅ |
| Google | ✅ | ✅ |
| Aliyun | ✅ | ✅ |
| Zhipu | ✅ | ✅ |
| Other | ✅ | ❌ |

## Examples

```bash
cargo run --example multimodal_basic
```
