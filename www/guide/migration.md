# Migration

## Migrating to v0.5.8

### Tencent Hunyuan Native API v3

In v0.5.8, the Tencent provider was updated to use the native Tencent Cloud API v3 with `TC3-HMAC-SHA256` signature.

**Breaking Change**:

- Old: `LlmClient::tencent("api-key")` (OpenAI compatible wrapper)
- New: `LlmClient::tencent("secret-id", "secret-key")` (Native API)

**Migration**:

```rust
// Before
let client = LlmClient::tencent("sk-...")?;

// After
let client = LlmClient::tencent("AKID...", "SecretKey...")?;
```

---

## Migrating to v0.5.0

### Multi-Modal Content

v0.5.0 introduced native multi-modal support, changing the `Message.content` field.

**Breaking Change**:

- `Message.content` is now `Vec<MessageBlock>` internally, though strictly speaking the public API might abstract this, direct field access may break.
- Helper methods like `Message::text()` should be used.

**Migration**:

```rust
// Before
let msg = Message { role: Role::User, content: "hello".to_string(), .. };

// After
let msg = Message::text(Role::User, "hello");
```
