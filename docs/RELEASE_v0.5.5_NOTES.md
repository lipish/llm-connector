# Release v0.5.5

## 🚀 Improvements

### Streaming Timeout Configuration
- **Increased default HTTP timeout**: 30s → 60s
  - Better support for long streaming responses
  - More reasonable default for LLM APIs
  - Reduces risk of premature timeouts

### Streaming Headers
- **Added SSE standard headers** for all streaming requests:
  - `Accept: text/event-stream`
  - `Cache-Control: no-cache`
  - `Connection: keep-alive`
- Ensures proper Server-Sent Events behavior
- Prevents caching issues with streaming responses

### Error Messages
- **Improved timeout error messages**
  - Now suggests increasing timeout for long-running streams
  - More actionable guidance for troubleshooting

### Code Quality
- **Complete internationalization**
  - All source code comments now in English
  - 100% Chinese to English translation complete
  - Zero Chinese characters remaining
  - 18 source files completely translated

## 🧪 Testing

### New Test Examples
- `examples/test_zhipu_streaming_timeout.rs` - Basic streaming test
- `examples/test_zhipu_long_streaming.rs` - Long response test (17s, 633 chunks)

### Verification
- ✅ All 82 tests passing
- ✅ Streaming verified with Zhipu GLM API
- ✅ Short responses: 2.27s, 52 chunks
- ✅ Long responses: 17.4s, 633 chunks
- ✅ No timeout issues observed

## 📚 Documentation

### New Documentation
- `docs/STREAMING_TIMEOUT_FIX.md` - Timeout improvements guide
- `docs/STREAMING_INVESTIGATION_REPORT.md` - Investigation results
- `docs/COMPLETE_CHINESE_CLEANUP_FINAL.md` - Translation summary
- `docs/CHINESE_CLEANUP_STATUS.md` - Translation progress

## 🔄 Migration

**Fully backward compatible** - No breaking changes.

All existing code continues to work without modification.

### Optional: Custom Timeout

For very long responses, you can now easily configure custom timeout:

```rust
// 120 seconds timeout for long responses
let client = LlmClient::zhipu_with_timeout(api_key, 120)?;
```

## 📦 Installation

```toml
[dependencies]
llm-connector = "0.5.5"
```

## 🔗 Links

- **Crates.io**: https://crates.io/crates/llm-connector/0.5.5
- **Documentation**: https://docs.rs/llm-connector/0.5.5
- **Repository**: https://github.com/lipish/llm-connector
- **Changelog**: https://github.com/lipish/llm-connector/blob/main/CHANGELOG.md

## 📊 Statistics

- **Files Modified**: 4 files
- **Lines Added**: 468 lines
- **Lines Removed**: 8 lines
- **Tests**: 82 passing
- **Build Time**: ~10 seconds
- **Package Size**: 215.5 KiB (compressed)

## 🙏 Acknowledgments

Thanks to all users who provided feedback on streaming behavior!

---

**Full Changelog**: https://github.com/lipish/llm-connector/compare/v0.5.4...v0.5.5

