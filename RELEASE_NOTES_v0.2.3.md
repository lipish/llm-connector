# Release Notes - v0.2.3

## 🎯 Summary

This release simplifies the API by removing the confusing `supported_models()` method and improves error messages for better developer experience. It also adds comprehensive debugging tools to help users troubleshoot API key issues.

## ✨ What's New

### 1. Simplified API - Removed `supported_models()`

**Why?**
- Returned empty `[]` for most protocols (OpenAI, Anthropic, Aliyun)
- Caused confusion - why have two methods (`supported_models()` and `fetch_models()`)?
- Users should use `fetch_models()` for real-time model discovery

**Migration:**
```rust
// ❌ Old (no longer works)
let models = client.supported_models();

// ✅ New
let models = client.fetch_models().await?;
```

### 2. Better Error Messages

**Before:**
```
Authentication failed: Incorrect API key provided: sk-78f43...bd03. 
You can find your API key at https://platform.openai.com/account/api-keys.
```

**After:**
```
Authentication failed: Incorrect API key provided: sk-78f43...bd03. 
Please verify your API key is correct and has the necessary permissions.
```

No more confusing OpenAI URLs when using DeepSeek, Zhipu, or other providers!

### 3. New Debugging Tools

**Test your API keys:**
```bash
cargo run --example test_keys_yaml
```

This will test all API keys in your `keys.yaml` and tell you exactly what's wrong!

**Debug DeepSeek specifically:**
```bash
cargo run --example debug_deepseek -- sk-your-key
```

**New Documentation:**
- `TROUBLESHOOTING.md` - Comprehensive troubleshooting guide
- `HOW_TO_TEST_YOUR_KEYS.md` - How to test your API keys
- `TEST_YOUR_DEEPSEEK_KEY.md` - Quick start for DeepSeek users

## 🔧 Breaking Changes

### Removed Methods

```rust
// ❌ No longer available:
client.supported_models()
provider.supports_model("model-name")
```

### How to Migrate

**Option 1: Use `fetch_models()`**
```rust
let models = client.fetch_models().await?;
println!("Available models: {:?}", models);
```

**Option 2: Just use any model name**
```rust
let request = ChatRequest {
    model: "deepseek-chat".to_string(),
    messages: vec![Message::user("Hello")],
    ..Default::default()
};
```

## 📦 Installation

```toml
[dependencies]
llm-connector = "0.2.3"
```

## ✅ Testing

All tests pass:
- 56 unit and integration tests
- All examples compile successfully
- No regressions

## 📝 Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for complete details.

## 🙏 Contributors

- @lipish

## 🔗 Links

- [Repository](https://github.com/lipish/llm-connector)
- [Crates.io](https://crates.io/crates/llm-connector)
- [Documentation](https://docs.rs/llm-connector)

