# Release v0.5.3 - Universal Reasoning Models Support

**Release Date**: 2025-01-15

## 🎉 New Features

### Universal Reasoning Models Support 🧠

llm-connector now provides **universal support for reasoning models** across all providers with zero configuration!

**Supported Models**:
- ✅ **Volcengine Doubao-Seed-Code** (`reasoning_content`)
- ✅ **DeepSeek R1** (`reasoning_content` / `reasoning`)
- ✅ **OpenAI o1 series** (`thought` / `reasoning_content`)
- ✅ **Qwen reasoning models** (`reasoning`)
- ✅ **Anthropic Claude extended thinking** (`thinking`)

**Key Benefits**:
- 🔧 **Zero Configuration**: Automatic field detection
- 🎯 **Unified Interface**: Same code for all reasoning models
- ✅ **Backward Compatible**: Standard models work as before
- 📊 **Priority-Based**: Standard `content` field takes precedence

**Usage Example**:
```rust
use futures_util::StreamExt;

// Works with ANY reasoning model!
let mut stream = provider.chat_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.get_content() {
        print!("{}", content);  // ✅ Automatically extracts reasoning content
    }
}
```

See [Reasoning Models Support Guide](../../REASONING_MODELS_SUPPORT.md) for details.

## 🐛 Bug Fixes

### Volcengine Streaming Support

**Fixed**: Volcengine streaming now correctly extracts content from reasoning models (Doubao-Seed-Code)

**Issue**: `StreamingResponse.get_content()` returned `None` for Doubao-Seed-Code model responses

**Root Cause**: Reasoning models output content in `delta.reasoning_content` field instead of `delta.content`

**Solution**: Enhanced SSE parser to check multiple content fields in priority order:
1. `delta.content` (standard OpenAI format, non-empty)
2. `delta.reasoning_content` (Volcengine Doubao-Seed-Code, DeepSeek R1)
3. `delta.reasoning` (Qwen, DeepSeek)
4. `delta.thought` (OpenAI o1)
5. `delta.thinking` (Anthropic)

**Test Results**:
- ✅ 221 tests passed
- ✅ Volcengine streaming: 101 chunks, 477 chars extracted
- ✅ All existing functionality preserved

## 📚 Documentation

### Documentation Structure Cleanup

**Reorganized**: Cleaned up docs directory from **52 to 30 files** (-42%)

**New Structure**:
```
docs/
├── Core documents (6): Architecture, migration guides, reasoning models support
├── Provider guides (7): Dedicated guide for each provider in docs/guides/
└── Archive (17): Historical releases and reports in docs/archive/
```

**New Provider Guides**:
- `docs/guides/ALIYUN_GUIDE.md` - Aliyun DashScope usage guide
- `docs/guides/ANTHROPIC_GUIDE.md` - Anthropic Claude usage guide
- `docs/guides/ZHIPU_GUIDE.md` - Zhipu GLM usage guide
- Updated existing guides for DeepSeek, Moonshot, Tencent, Volcengine

**Improvements**:
- ✅ Clear documentation index in `docs/README.md`
- ✅ Removed duplicate and outdated content
- ✅ Better organization and discoverability

## 🔒 Security

### Sensitive Information Obfuscation

**Obfuscated**: All sensitive information in documentation and examples
- API keys replaced with placeholders
- Endpoint IDs replaced with example values
- Created `keys.yaml.example` for configuration reference

See `docs/SENSITIVE_INFO_OBFUSCATION.md` for details.

## 📦 What's Changed

### Core Changes
- Enhanced SSE parser in `src/sse.rs` for reasoning models support
- Added unit test: `test_streaming_response_content_population`

### Documentation
- Reorganized docs directory structure
- Added comprehensive provider guides
- Created reasoning models support guide

### Examples
- Added `examples/volcengine_streaming.rs` - Volcengine streaming test example
- Added `scripts/test_volcengine_streaming.sh` - Automation script

### Configuration
- Created `keys.yaml.example` - Example configuration file

## 🔗 Links

- **Crates.io**: https://crates.io/crates/llm-connector/0.5.3
- **Documentation**: https://docs.rs/llm-connector/0.5.3
- **Repository**: https://github.com/lipish/llm-connector
- **Changelog**: [CHANGELOG.md](../../../CHANGELOG.md)

## 📊 Statistics

- **Files Changed**: 61 files
- **Insertions**: +1,791 lines
- **Deletions**: -6,768 lines
- **Tests**: 221 tests passing
- **Documentation**: 30 files (from 52)

## 🙏 Acknowledgments

Thanks to all users who reported issues and provided feedback on reasoning models support!

---

**Full Changelog**: https://github.com/lipish/llm-connector/compare/v0.5.2...v0.5.3

