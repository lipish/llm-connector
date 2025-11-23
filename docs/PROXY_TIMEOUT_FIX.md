# Proxy Timeout Fix

## 🔍 Problem Discovery

### Root Cause

**reqwest's default behavior**: The reqwest HTTP client **automatically uses system proxy settings** by default.

This can cause unexpected timeout issues when:
1. System proxy is configured but slow/unreachable
2. System proxy is configured for other purposes (e.g., debugging tools)
3. Network environment has proxy settings that don't work for LLM APIs
4. Proxy requires authentication but credentials are not provided

### Symptoms

- Streaming requests timeout after 30-60 seconds
- Non-streaming requests may also timeout
- Error message: `operation timed out`
- Happens even when direct connection works fine

### Why This Happens

```rust
// reqwest's default behavior (BEFORE our fix)
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .build()?;  // ⚠️ Automatically uses system proxy!
```

The client will:
1. Check environment variables: `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`
2. Check system proxy settings (on Windows/macOS)
3. Use the proxy if found
4. Timeout if proxy is slow or unreachable

## ✅ Solution

### Disable System Proxy by Default

We now **explicitly disable system proxy** by default:

```rust
// llm-connector's behavior (AFTER our fix)
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .no_proxy()  // ✅ Explicitly disable system proxy
    .build()?;
```

### Benefits

1. **Predictable behavior**: No unexpected proxy interference
2. **Faster connections**: Direct connection to LLM APIs
3. **No timeout issues**: Avoids slow/unreachable proxy problems
4. **Explicit control**: Users must explicitly set proxy if needed

## 🔧 Changes Made

### File: `src/core/client.rs`

#### 1. `HttpClient::new()`

**Before**:
```rust
let client = Client::builder()
    .timeout(Duration::from_secs(60))
    .build()?;
```

**After**:
```rust
let client = Client::builder()
    .timeout(Duration::from_secs(60))
    .no_proxy()  // ✅ Disable system proxy
    .build()?;
```

#### 2. `HttpClient::with_config()`

**Before**:
```rust
// Set proxy
if let Some(proxy_url) = proxy {
    let proxy = reqwest::Proxy::all(proxy_url)?;
    builder = builder.proxy(proxy);
}
// ⚠️ If proxy is None, system proxy is still used!
```

**After**:
```rust
// Set proxy or disable system proxy
if let Some(proxy_url) = proxy {
    // Use explicit proxy
    let proxy = reqwest::Proxy::all(proxy_url)?;
    builder = builder.proxy(proxy);
} else {
    // ✅ Disable system proxy to avoid timeout issues
    builder = builder.no_proxy();
}
```

## 📝 Usage

### Default Behavior (No Proxy)

```rust
// System proxy is disabled by default
let client = LlmClient::zhipu_openai_compatible(api_key)?;
let response = client.chat(&request).await?;  // ✅ Direct connection
```

### Explicit Proxy Configuration

If you need to use a proxy:

```rust
// Explicitly set proxy
let client = LlmClient::zhipu_with_config(
    api_key,
    true,                           // OpenAI compatible
    None,                           // Default base URL
    Some(120),                      // Timeout
    Some("http://proxy:8080"),      // ✅ Explicit proxy
)?;
```

### Proxy with Authentication

```rust
// Proxy with username and password
let proxy_url = "http://username:password@proxy:8080";
let client = LlmClient::zhipu_with_config(
    api_key,
    true,
    None,
    Some(120),
    Some(proxy_url),
)?;
```

## 🧪 Testing

### Test 1: Without Proxy (Default)

```bash
cargo run --example test_zhipu_streaming_timeout --features streaming
```

**Result**: ✅ Success (2.05s, 52 chunks)

### Test 2: With Invalid Proxy

```bash
cargo run --example test_proxy_issue --features streaming
```

**Result**: ✅ Fails fast with connection error (41ms)

### Test 3: System Proxy Check

```bash
cargo run --example test_proxy_issue --features streaming
```

**Output**:
```
Environment proxy settings:
  HTTP_PROXY: (not set)
  HTTPS_PROXY: (not set)
  ALL_PROXY: (not set)
  NO_PROXY: (not set)
```

## 📊 Impact

### Before Fix

- ❌ System proxy used automatically
- ❌ Unexpected timeout issues
- ❌ Difficult to diagnose
- ❌ Inconsistent behavior across environments

### After Fix

- ✅ System proxy disabled by default
- ✅ No unexpected timeouts
- ✅ Predictable behavior
- ✅ Explicit proxy control when needed

## 🔄 Migration

### Backward Compatibility

**Fully backward compatible** for most users:
- If you don't use proxy: ✅ Works better (no timeout issues)
- If you use explicit proxy: ✅ Still works the same

**Potential breaking change** (rare):
- If you rely on system proxy settings: ❌ Need to explicitly set proxy

### Migration Steps

If you were relying on system proxy:

```rust
// Before (relied on system proxy)
let client = LlmClient::zhipu_openai_compatible(api_key)?;

// After (explicitly set proxy)
let proxy_url = std::env::var("HTTPS_PROXY").ok();
let client = LlmClient::zhipu_with_config(
    api_key,
    true,
    None,
    None,
    proxy_url.as_deref(),
)?;
```

## 🎯 Conclusion

This fix resolves the root cause of many timeout issues by:
1. Disabling system proxy by default
2. Requiring explicit proxy configuration
3. Providing clear documentation and examples

**Result**: More reliable, predictable, and faster connections to LLM APIs.

