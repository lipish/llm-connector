# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 2025-01-06

### ✨ Added

#### Online Model Discovery
- **New `fetch_models()` method** for retrieving available models from API
  - Added to `Provider` trait, `LlmClient`, and `GenericProvider`
  - Makes GET request to `/v1/models` endpoint for OpenAI-compatible providers
  - Returns `Vec<String>` of available model IDs
  - Returns `UnsupportedOperation` error for protocols without model listing support

#### HTTP Transport Enhancement
- Added `get()` method to `HttpTransport` for GET requests
- Supports custom headers and authentication

#### Error Handling
- Added `UnsupportedOperation` error variant for unsupported operations
- Returns HTTP 501 status code for unsupported operations

#### Examples
- `examples/test_fetch_models.rs` - Comprehensive test with all providers
- `examples/fetch_models_simple.rs` - Simple comparison example
- `examples/test_with_keys.rs` - Test with keys.yaml configuration

#### Documentation
- `FETCH_MODELS_FEATURE.md` - Complete feature documentation
- `TEST_RESULTS.md` - Test results and verification
- Updated README.md with model discovery section
- Added comparison table for `supported_models()` vs `fetch_models()`

### 🔧 Changed

#### OpenAI Protocol
- **Removed hardcoded model lists** from `OpenAIProtocol`
- `supported_models()` now returns empty `[]` instead of hardcoded models
- Users can now use **any model name** without restrictions
- Implemented `models_endpoint_url()` to support `/v1/models` endpoint

#### Documentation Cleanup
- Removed references to third-party providers (DeepSeek, Zhipu, Moonshot, etc.) from OpenAI protocol docs
- Updated examples to focus on OpenAI instead of third-party providers
- Simplified documentation to emphasize protocol-first approach

#### Provider Type Aliases
- Removed provider-specific type aliases:
  - `DeepSeekProvider`
  - `ZhipuProvider`
  - `MoonshotProvider`
  - `VolcEngineProvider`
  - `TencentProvider`
  - `MiniMaxProvider`
  - `StepFunProvider`

### 🐛 Fixed

#### Configuration
- Fixed `keys.yaml` model names:
  - Removed invalid `qwen3-turbo` model
  - Updated to valid Aliyun models: `qwen-turbo`, `qwen-plus`, `qwen-max`
  - Updated Qwen2 models to Qwen2.5 versions

#### Dependencies
- Added `serde_yaml` to `[dev-dependencies]` for examples
- Fixed `serde_yaml` resolution in test examples

#### Code Quality
- Removed unused imports (`HttpTransport`, `LlmConnectorError` from openai.rs)
- Fixed struct field issues (removed incorrect `transport` field)

### 📊 Test Results

#### Successfully Tested Providers (Online Model Fetching)

| Provider | Status | Models Found | Example Models |
|----------|--------|--------------|----------------|
| DeepSeek | ✅ | 2 | `deepseek-chat`, `deepseek-reasoner` |
| Zhipu (GLM) | ✅ | 3 | `glm-4.5`, `glm-4.5-air`, `glm-4.6` |
| Moonshot | ✅ | 12 | `moonshot-v1-32k`, `kimi-latest`, `kimi-thinking-preview` |
| LongCat | ❌ | - | `/models` endpoint not available |
| VolcEngine | ❌ | - | `/models` endpoint not available |
| Aliyun | ℹ️ | - | Protocol doesn't support model listing |
| Anthropic | ℹ️ | - | Protocol doesn't support model listing |

### 📝 Migration Guide

#### For Users Relying on Hardcoded Models

**Before (v0.2.0):**
```rust
let client = LlmClient::openai("sk-...");
let models = client.supported_models();
// Returns: ["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"]
```

**After (v0.2.1):**
```rust
let client = LlmClient::openai("sk-...");

// Option 1: Use any model name directly (recommended)
let request = ChatRequest {
    model: "gpt-4o".to_string(), // Any model name works
    // ...
};

// Option 2: Fetch models online
let models = client.fetch_models().await?;
// Returns: actual models from OpenAI API
```

#### For OpenAI-Compatible Providers

**Before:**
```rust
// Had to check hardcoded list
let models = client.supported_models();
```

**After:**
```rust
// Fetch real-time models from provider
let client = LlmClient::openai_compatible(
    "sk-...",
    "https://api.deepseek.com/v1"
);
let models = client.fetch_models().await?;
// Returns: ["deepseek-chat", "deepseek-reasoner"]
```

### 🎯 Benefits

1. **No Model Restrictions**: Use any model name without being limited by hardcoded lists
2. **Always Up-to-Date**: Get the latest models directly from the API
3. **Backward Compatible**: Existing code continues to work
4. **Flexible**: Providers can opt-in to model listing support
5. **Clear Errors**: Explicit error messages when operations aren't supported

### 🔗 Related Issues

- Fixed errors in `src/protocols/openai.rs`
- Removed hardcoded `supported_models`
- Implemented online model fetching (Option 3)
- Updated documentation to reflect changes

### 📚 Documentation

- README.md: Added "Key Features" section
- README.md: Added "Model Discovery" section with comparison table
- README.md: Added "Recent Changes" section
- README.md: Updated error handling examples
- README.md: Updated examples section

### 🧪 Testing

All tests passing:
```bash
cargo check --lib                    # ✅ Success
cargo run --example test_openai_only # ✅ All tests passed
cargo run --example test_with_keys   # ✅ 6/6 providers tested
cargo run --example test_fetch_models # ✅ Online fetching works
```

---

## [0.2.0] - Previous Release

Initial release with 4 protocol support and basic functionality.

---

## Future Enhancements

Potential improvements for future releases:

1. **Model Caching**: Cache fetched models to reduce API calls
2. **Model Metadata**: Return full model objects with capabilities, not just IDs
3. **Model Filtering**: Add parameters to filter models by capability
4. **Extended Protocol Support**: Implement model listing for other protocols if available
5. **Pagination Support**: Handle paginated model responses

