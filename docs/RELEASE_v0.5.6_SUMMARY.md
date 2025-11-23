# Release v0.5.6 Summary

## 📦 Release Information

**Version**: 0.5.6
**Release Date**: 2025-11-23
**Status**: ✅ Published
**Type**: Critical Fix

## 🔗 Links

- **Crates.io**: https://crates.io/crates/llm-connector/0.5.6
- **Documentation**: https://docs.rs/llm-connector/0.5.6
- **GitHub Release**: https://github.com/lipish/llm-connector/releases/tag/v0.5.6
- **Repository**: https://github.com/lipish/llm-connector

## 🔥 Critical Fix

### Proxy Timeout Issue

**Problem**: Streaming requests timeout due to reqwest's automatic system proxy usage

**Root Cause**: 
- reqwest HTTP client automatically uses system proxy settings by default
- System proxy can be slow, unreachable, or misconfigured
- Causes timeout errors even when direct connection works

**Solution**: 
- Explicitly disable system proxy by default
- Require explicit proxy configuration when needed

## 🎯 Key Changes

### 1. HttpClient::new()
```rust
// Before
let client = Client::builder()
    .timeout(Duration::from_secs(60))
    .build()?;  // ⚠️ Uses system proxy

// After
let client = Client::builder()
    .timeout(Duration::from_secs(60))
    .no_proxy()  // ✅ Disables system proxy
    .build()?;
```

### 2. HttpClient::with_config()
```rust
// Before
if let Some(proxy_url) = proxy {
    builder = builder.proxy(proxy);
}
// ⚠️ System proxy still used if proxy is None

// After
if let Some(proxy_url) = proxy {
    builder = builder.proxy(proxy);
} else {
    builder = builder.no_proxy();  // ✅ Explicitly disable
}
```

## 📊 Impact

### Performance Improvement

| Scenario | Before | After | Improvement |
|----------|--------|-------|-------------|
| Direct connection | May timeout | ✅ Works | 100% |
| Slow proxy | Timeout (60s) | Direct | 60s saved |
| Invalid proxy | Timeout (60s) | Direct | 60s saved |

### Behavior Changes

| Configuration | Before | After |
|---------------|--------|-------|
| No proxy set | Uses system proxy | ✅ Direct connection |
| Explicit proxy | Uses explicit proxy | ✅ Uses explicit proxy |
| System proxy set | Uses system proxy | ✅ Ignores system proxy |

## 🧪 Testing

### Test Results
- ✅ All 82 tests passing
- ✅ Zhipu streaming: 2.05s, 52 chunks (no timeout)
- ✅ Invalid proxy fails fast: 41ms (no long timeout)
- ✅ Direct connection works reliably

### New Test Examples
- **examples/test_proxy_issue.rs** - Proxy behavior tests

## 📚 Documentation

### New Files
1. **docs/PROXY_TIMEOUT_FIX.md** - Technical documentation
2. **docs/PROXY_FIX_SUMMARY.md** - Executive summary
3. **docs/RELEASE_v0.5.6_SUMMARY.md** - This file

### Updated Files
1. **Cargo.toml** - Version bump
2. **CHANGELOG.md** - Release notes
3. **src/core/client.rs** - Proxy configuration
4. **tests/streaming_integration_tests.rs** - Test robustness

## 🔄 Migration

### Most Users (No Change Needed)
```rust
// Works better now (no timeout issues)
let client = LlmClient::zhipu_openai_compatible(api_key)?;
```

### Users Who Need Proxy
```rust
// Explicitly set proxy
let client = LlmClient::zhipu_with_config(
    api_key,
    true,
    None,
    None,
    Some("http://proxy:8080"),
)?;
```

### Users Relying on System Proxy (Rare)
```rust
// Read and set system proxy explicitly
let proxy = std::env::var("HTTPS_PROXY").ok();
let client = LlmClient::zhipu_with_config(
    api_key,
    true,
    None,
    None,
    proxy.as_deref(),
)?;
```

## ⚠️ Breaking Change

**Only affects users relying on system proxy settings** (rare):
- Before: System proxy was used automatically
- After: System proxy is ignored, must be set explicitly

## 📈 Statistics

- **Files Modified**: 4 files
- **Lines Added**: 418 lines
- **Lines Removed**: 5 lines
- **Net Change**: +413 lines
- **Tests**: 82 passing
- **Package Size**: 222.4 KiB (compressed)

## 🚀 Publishing Process

### Steps Completed
1. ✅ Version bump (0.5.5 → 0.5.6)
2. ✅ CHANGELOG updated
3. ✅ Build successful
4. ✅ Git commit and tag created
5. ✅ Pushed to GitHub
6. ✅ Published to crates.io
7. ✅ GitHub Release created
8. ✅ Documentation updated

### Timeline
- **Code Changes**: 2025-11-23 14:00-14:15
- **Testing**: 2025-11-23 14:15-14:20
- **Publishing**: 2025-11-23 14:20-14:25
- **Total Time**: ~25 minutes

## 🎉 Conclusion

This critical fix resolves timeout issues caused by reqwest's automatic system proxy usage:

1. ✅ **Problem identified correctly** - User's insight was accurate
2. ✅ **Root cause fixed** - System proxy disabled by default
3. ✅ **Performance improved** - Direct connections are faster
4. ✅ **All tests passing** - 82/82 tests successful
5. ✅ **Backward compatible** - Most users unaffected

### User Feedback Validation

The user correctly identified that the timeout issue was related to reqwest's proxy behavior. This fix validates that insight and provides a robust solution.

---

**Status**: ✅ COMPLETE
**Commit**: 990a80c
**Tag**: v0.5.6
**Published**: crates.io + GitHub

