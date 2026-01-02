
### ⚠️ Breaking Changes

#### Tencent Hunyuan Native API v3
- **BREAKING**: Replaced OpenAI-compatible wrapper with native Tencent Cloud API v3 using `TC3-HMAC-SHA256` signature.
- **Affected**: `LlmClient::tencent()` and `tencent()` provider functions.
- **New Signature**: `tencent(secret_id, secret_key)` (previously `tencent(api_key)`).
- **Rationale**: Support native signature verification for better security and stability.

### ✨ Improvements

- **Security**: Hardcoded API keys removed from documentation and code.
- **Documentation**: Updated Tencent guide with native API usage.
