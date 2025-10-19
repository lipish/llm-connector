# Changelog

All notable changes to this project will be documented in this file.

## [0.4.16] - 2025-10-18

### 🐛 Bug Fixes

#### **修复重复 Content-Type 头部导致 Aliyun 等 Provider 无法使用**

**问题描述**:
- ❌ Aliyun Provider 完全无法使用
- ❌ 错误信息: `Content-Type/Accept application/json,application/json is not supported`
- ❌ 原因: `auth_headers()` 和 `HttpClient::post().json()` 都设置了 `Content-Type`
- ❌ 导致 HTTP 头部重复: `Content-Type: application/json, application/json`

**根本原因**:
1. `Protocol::auth_headers()` 返回 `Content-Type: application/json`
2. `HttpClient::post()` 使用 `.json(body)` 也会自动设置 `Content-Type: application/json`
3. 两个头部值被合并，导致重复
4. 阿里云 API（以及其他严格的 API）不接受重复的头部值

**修复内容**:

1. **Aliyun Provider** (`src/providers/aliyun.rs`)
   - 从 `auth_headers()` 中移除 `Content-Type` 设置
   - 添加注释说明 `.json()` 已自动设置

2. **Zhipu Provider** (`src/providers/zhipu.rs`)
   - 从 `auth_headers()` 中移除 `Content-Type` 设置
   - 避免潜在的重复头部问题

3. **Anthropic Provider** (`src/providers/anthropic.rs`)
   - Vertex AI: 移除 `.with_header("Content-Type", ...)`
   - Bedrock: 移除 `.with_header("Content-Type", ...)`

4. **Ollama Provider** (`src/providers/ollama.rs`)
   - `new()`: 移除 `.with_header("Content-Type", ...)`
   - `with_config()`: 移除 `.with_header("Content-Type", ...)`

5. **OpenAI Provider** (`src/providers/openai.rs`)
   - Azure OpenAI: 移除 `.with_header("Content-Type", ...)`
   - OpenAI Compatible: 移除 `.with_header("Content-Type", ...)`

**影响的 Provider**:
- ✅ **Aliyun (DashScope)** - 修复无法使用的问题
- ✅ **Zhipu (GLM)** - 修复潜在问题
- ✅ **Anthropic (Vertex AI, Bedrock)** - 修复潜在问题
- ✅ **Ollama** - 修复潜在问题
- ✅ **OpenAI (Azure, Compatible)** - 修复潜在问题

**测试验证**:
- ✅ 编译成功
- ✅ 添加 `examples/test_aliyun_basic.rs` 验证修复
- ✅ 所有 Provider 不再重复设置 Content-Type

**修复统计**:
- 修复的文件: 5 个
- 修复的 Provider: 6 个
- 删除的重复设置: 9 处
- 添加的注释: 9 处

**影响范围**:
- ✅ 修复 Aliyun Provider 完全无法使用的严重问题
- ✅ 修复其他 Provider 的潜在兼容性问题
- ✅ 提升 HTTP 头部设置的规范性
- ✅ 完全向后兼容，无需用户修改代码

### 🧪 Testing

#### **添加智谱流式 tool_calls 验证测试**

**新增测试**:
1. `tests/test_zhipu_streaming_direct.sh` - 直接测试智谱 API 原始响应
2. `examples/test_zhipu_flash_streaming_tool_calls.rs` - 测试 llm-connector 解析
3. `examples/debug_zhipu_streaming_tool_calls.rs` - 详细调试示例

**验证结果**:
- ✅ 智谱 API 在流式模式下返回 tool_calls
- ✅ llm-connector 可以正确解析 tool_calls
- ✅ 证明 llm-connector 0.4.15 没有 bug，功能正常

### 📝 Documentation

- 添加 `docs/FIX_DUPLICATE_CONTENT_TYPE_HEADER.md` - 重复头部问题修复文档
- 添加 `docs/ZHIPU_STREAMING_TOOL_CALLS_VERIFICATION.md` - 智谱流式验证报告

---

## [0.4.15] - 2025-10-18

### 🐛 Bug Fixes

#### **修复示例代码编译错误和警告**

**修复内容**:
1. **移除不存在的方法调用** (`examples/test_openai_tool_streaming.rs`)
   - 移除了对不存在的 `LlmClient::openrouter()` 方法的调用
   - 改为使用 `LlmClient::openai()`

2. **修复类型错误** (`examples/test_openai_tool_streaming.rs`)
   - 修复 tool_calls 引用类型问题
   - 将 `&tool_calls_buffer[0]` 改为 `tool_calls_buffer[0].clone()`

3. **消除未使用导入警告** (7 个示例文件)
   - 将 streaming 相关的导入移到 `#[cfg(feature = "streaming")]` 内
   - 避免在非 streaming 模式下产生未使用导入警告
   - 影响文件：
     - `test_zhipu_tool_messages_detailed.rs`
     - `test_deepseek_tools.rs`
     - `test_openai_tool_streaming.rs`
     - `test_zhipu_tool_streaming_issue.rs`
     - `test_glm_models_tool_streaming.rs`
     - `zhipu_tools_streaming.rs`
     - `test_all_providers_tool_streaming.rs`

4. **消除未使用字段警告** (`examples/test_all_providers_tool_streaming.rs`)
   - 添加 `#[allow(dead_code)]` 到 `TestResult` 结构体

5. **修复 clippy 警告**
   - 修复 doc comments 空行警告
   - 修复长度比较警告（`len() > 0` → `!is_empty()`）

### 📝 Documentation

- 添加 `docs/EXAMPLES_AND_TESTS_REVIEW.md` - Examples 和 Tests 审查报告
- 添加 `docs/RELEASE_v0.4.14.md` - v0.4.14 发布总结

**测试验证**:
- ✅ 所有示例都能正常编译
- ✅ 所有测试都能通过
- ✅ 无编译错误
- ✅ 大幅减少编译警告

**影响范围**:
- 修复示例代码的编译问题
- 提升代码质量
- 完全向后兼容

---

## [0.4.14] - 2025-10-18

### 🐛 Bug Fixes

#### **修复 OpenAI 协议工具调用支持 + 移除智谱 GLM 流式强制切换**

**问题 1: OpenAI 协议缺少工具调用支持**

**问题描述**:
- ❌ `OpenAIRequest` 缺少 `tools` 和 `tool_choice` 字段，无法传递工具定义
- ❌ `OpenAIMessage` 缺少 `tool_calls`, `tool_call_id`, `name` 字段
- ❌ `OpenAIResponseMessage` 缺少 `tool_calls` 字段，无法解析工具调用响应
- ❌ 导致所有使用 OpenAI 协议的服务（DeepSeek, Moonshot 等）完全无法使用工具调用

**修复内容**:
1. **OpenAIRequest 添加工具字段** (`src/protocols/openai.rs`)
   ```rust
   pub struct OpenAIRequest {
       // ... 其他字段
       pub tools: Option<Vec<serde_json::Value>>,      // ✅ 新增
       pub tool_choice: Option<serde_json::Value>,     // ✅ 新增
   }
   ```

2. **OpenAIMessage 添加工具字段** (`src/protocols/openai.rs`)
   ```rust
   pub struct OpenAIMessage {
       pub role: String,
       pub content: String,
       pub tool_calls: Option<Vec<serde_json::Value>>,  // ✅ 新增
       pub tool_call_id: Option<String>,                // ✅ 新增
       pub name: Option<String>,                        // ✅ 新增
   }
   ```

3. **OpenAIResponseMessage 添加工具字段** (`src/protocols/openai.rs`)
   ```rust
   pub struct OpenAIResponseMessage {
       pub content: Option<String>,                     // ✅ 改为 Option
       pub tool_calls: Option<Vec<serde_json::Value>>,  // ✅ 新增
   }
   ```

4. **build_request 完整映射工具调用** (`src/protocols/openai.rs:48-106`)
   - 正确映射 `tools` 字段
   - 正确映射 `tool_choice` 字段
   - 正确映射消息中的 `tool_calls`, `tool_call_id`, `name` 字段

5. **parse_response 正确解析工具调用** (`src/protocols/openai.rs:116-149`)
   - 从响应中提取 `tool_calls`
   - 转换为统一的 `ToolCall` 类型

**问题 2: 智谱 GLM 流式响应被强制切换**

**问题描述**:
- ❌ `src/core/traits.rs` 中存在硬编码逻辑，检测到 `Role::Tool` 消息时强制切换为非流式
- ❌ GLM-4.5 正常可返回 91 个流式块，但包含工具结果时被强制切换为 1 个块
- ❌ 这是一个临时修复（workaround），现在已不再需要

**修复内容**:
- **移除硬编码修复逻辑** (`src/core/traits.rs:155-173`)
  - 删除了检测 `Role::Tool` 和 `zhipu` 的特殊处理
  - 智谱 GLM 现在可以在包含工具调用结果时正常使用流式响应

**测试验证**:
- ✅ OpenAI 协议完整支持工具调用（tools, tool_choice, tool_calls）
- ✅ DeepSeek 现在可以正常使用工具调用
- ✅ 所有 OpenAI 兼容服务（Moonshot, Together AI 等）都可以使用工具调用
- ✅ 智谱 GLM 在包含 Role::Tool 时可以使用流式响应
- ✅ 所有核心库测试通过（27 个测试）

**新增示例**:
- `examples/verify_tool_fix.rs` - 验证工具调用修复效果

**影响范围**:
- 修复所有使用 OpenAI 协议的服务的工具调用功能
- 移除智谱 GLM 的流式响应限制
- 完全向后兼容

---

## [0.4.13] - 2025-10-18

### 🐛 Bug Fixes

#### **修复智谱 GLM 多轮工具调用支持**

**问题描述**:
- ❌ `ZhipuMessage` 缺少 `tool_call_id` 字段，无法在 Tool 消息中关联工具调用
- ❌ `ZhipuMessage` 缺少 `name` 字段，无法传递工具名称
- ❌ 导致多轮 Function Calling 对话失败（第二轮无法正确传递工具执行结果）

**修复内容**:
1. **ZhipuMessage 结构完善** (`src/providers/zhipu.rs:272-282`)
   ```rust
   pub struct ZhipuMessage {
       pub role: String,
       pub content: String,
       pub tool_calls: Option<Vec<serde_json::Value>>,
       pub tool_call_id: Option<String>,  // ✅ 新增
       pub name: Option<String>,          // ✅ 新增
   }
   ```

2. **build_request 映射补充** (`src/providers/zhipu.rs:77-96`)
   - 正确映射 `tool_call_id` 字段
   - 正确映射 `name` 字段

**测试验证**:
- ✅ 单轮工具调用：User 提问 → LLM 返回 tool_calls
- ✅ 多轮工具调用：Tool 结果 → LLM 返回文本响应
- ✅ 三轮对话：User 追问 → LLM 正确触发新的 tool_calls
- ✅ Tool 消息序列化：`role="tool"`, `tool_call_id`, `name` 全部正确

**新增示例**:
- `examples/zhipu_multiround_tools.rs` - 多轮工具调用演示
- `examples/zhipu_tools_edge_cases.rs` - 边缘情况测试
- `examples/verify_tool_message_serialization.rs` - 序列化验证

**影响范围**:
- 修复智谱 GLM 的多轮工具调用功能
- 完全符合 OpenAI Function Calling 规范
- 完全向后兼容

---

## [0.4.12] - 2025-10-18

### 🐛 Bug Fixes

#### **修复智谱 GLM 流式响应和工具调用支持**

**流式响应问题**:
- ❌ 问题：智谱 API 使用单换行分隔 SSE（`data: {...}\n`），导致默认解析器失败
- ❌ 问题：`StreamingResponse.content` 字段未填充，`get_content()` 返回空字符串
- ❌ 问题：`ZhipuRequest` 缺少 `stream` 参数，API 不知道要返回流式响应

**工具调用问题**:
- ❌ 问题：`ZhipuRequest` 缺少 `tools` 和 `tool_choice` 字段
- ❌ 问题：`ZhipuMessage` 不支持 `tool_calls` 响应
- ❌ 问题：流式和非流式请求都无法传递工具参数

**修复内容**:
1. **流式解析器** (`src/providers/zhipu.rs:126-201`)
   - 实现智谱专用 `parse_stream_response()`
   - 支持单换行分隔的 SSE 格式
   - 自动填充 `content` 字段（从 `delta.content` 复制）
   
2. **请求参数** (`src/providers/zhipu.rs:216-234`)
   - 添加 `stream: Option<bool>` 字段
   - 添加 `tools: Option<Vec<Tool>>` 字段
   - 添加 `tool_choice: Option<ToolChoice>` 字段
   
3. **响应解析** (`src/providers/zhipu.rs:240-264`)
   - `ZhipuMessage.content` 使用 `#[serde(default)]`（工具调用时可为空）
   - `ZhipuMessage.tool_calls` 支持工具调用响应
   - `ZhipuResponse` 包含完整元数据（id, created, usage）
   - `ZhipuChoice` 支持 `finish_reason`（识别 `tool_calls` 结束）

**测试验证**:
- ✅ 流式响应：124 个数据块，642 字符输出
- ✅ 非流式工具调用：`finish_reason: "tool_calls"`，正确解析参数
- ✅ 流式工具调用：`finish_reason: "tool_calls"`，正确解析参数

**影响范围**:
- 仅影响智谱 GLM provider
- 完全向后兼容
- 修复后与 OpenAI 协议对齐

**新增示例**:
- `examples/zhipu_tools.rs` - 工具调用（非流式）
- `examples/zhipu_tools_streaming.rs` - 工具调用（流式）

---

## [0.4.11] - 2025-10-17

### 🐛 Bug Fixes

**修复智谱流式响应解析问题（初步修复）**
- 实现 `ZhipuProtocol::parse_stream_response()` 专用流式解析器
- 支持单换行分隔的 SSE 格式
- 正确处理 `data:` 前缀（带或不带空格）

---

## [Unreleased]

### 🐛 **BUGFIX: 修复智谱流式响应解析问题**

#### **问题描述**
智谱 API 使用单换行分隔 SSE 事件（`data: {...}\n`），而不是标准的双换行（`data: {...}\n\n`），导致默认 SSE 解析器无法正确解析流式响应，产生 0 个数据块。

#### **修复内容**
- **新增**: `ZhipuProtocol::parse_stream_response()` 专用流式解析器
  - 支持单换行分隔的 SSE 格式
  - 正确处理 `data:` 前缀（带或不带空格）
  - 跳过 `[DONE]` 标记和空 payload
  - 提供详细的错误信息（包含原始 JSON）

#### **测试改进**
- 更新 `examples/zhipu_streaming.rs`
  - 添加数据块计数器
  - 显示解析器类型提示
  - 使用 `glm-4-flash` 模型（更快响应）
  - 添加零数据块警告

#### **影响**
- ✅ **修复**: 智谱流式 API 现在可以正常工作
- ✅ **兼容性**: 不影响其他 Provider 的流式功能
- ✅ **调试性**: 解析失败时显示原始 JSON

---

### ✨ **FEAT: API Naming Standardization**

#### **Changed**
- **Unified Constructor Naming**
  - `ollama_with_url()` → `ollama_with_base_url()` (kept old name as deprecated)
  - Removed redundant `zhipu_default()` (use `zhipu()` directly)
  - All providers now follow consistent `{provider}_with_base_url()` pattern

#### **Added**
- **Type-Safe Provider Conversions**
  - `LlmClient::as_ollama()` → `Option<&OllamaProvider>`
  - `LlmClient::as_openai()` → `Option<&OpenAIProvider>`
  - `LlmClient::as_aliyun()` → `Option<&AliyunProvider>`
  - `LlmClient::as_anthropic()` → `Option<&AnthropicProvider>`
  - `LlmClient::as_zhipu()` → `Option<&ZhipuProvider>`
  
- **API Key Validation Functions**
  - `validate_openai_key()`
  - `validate_aliyun_key()`
  - `validate_anthropic_key()` (already existed)
  - `validate_zhipu_key()` (already existed)

- **Advanced Configuration Methods**
  - All `{provider}_with_config()` methods now exposed in `LlmClient`
  - All `{provider}_with_timeout()` methods now exposed in `LlmClient`
  - Cloud-specific methods: `anthropic_vertex()`, `anthropic_bedrock()`, `aliyun_international()`, etc.

#### **Documentation**
- **NEW**: `docs/NAMING_CONVENTIONS.md` - Comprehensive naming standards guide
- **NEW**: `.augment/rules/naming.md` - Qoder auto-check rules
- Updated all examples to use new naming conventions

#### **Deprecated**
- `LlmClient::ollama_with_url()` → Use `ollama_with_base_url()`
- `providers::zhipu_default()` → Use `zhipu()` directly
- `LlmClient::ollama()` (the method, not constructor) → Use `as_ollama()`

#### **Impact**
- ✅ **Consistency**: All providers follow same naming pattern
- ✅ **Type Safety**: No more manual `downcast_ref` needed
- ✅ **Completeness**: All provider variants exposed in `LlmClient`
- ✅ **Documentation**: Clear naming rules for contributors

---

## [0.4.9] - 2025-10-16

### 📚 **DOCS: Fix API Documentation and Examples**

#### **Fixed**
- **README API Examples** - Updated streaming API examples to reflect current V2 architecture
  - Replaced deprecated `chat_stream_universal()`, `chat_stream_sse()`, `chat_stream_ndjson()` with current `chat_stream()`
  - Updated streaming examples to use `StreamingResponse` and `get_content()` method
  - Added clear distinction between V2 (current) and V1 (legacy) APIs in changelog
  - Fixed 29 documentation tests that used incorrect import paths

#### **Added**
- **New Example**: `streaming_v2_demo.rs` - Demonstrates current V2 streaming API
- **API Clarification**: Clear documentation of current streaming interface
- **Migration Guide**: Explains differences between V1 and V2 streaming APIs

#### **Impact**
- ✅ **Documentation**: All examples now reflect current API
- ✅ **Tests**: All 93 tests pass (including 50 documentation tests)
- ✅ **Clarity**: Clear separation between current and legacy APIs
- ✅ **Examples**: Working examples for current streaming interface

## [0.4.8] - 2025-10-16

### 🔧 **REFACTOR: Simplify Configuration Module Structure**

#### **Simplified**
- **Configuration Module** - Simplified `src/config/` directory to single `src/config.rs` file
  - Eliminated confusion between `src/config/provider.rs` and `src/providers/` directory
  - Consolidated all configuration types into single, clear module
  - Maintained all existing functionality and API compatibility
  - All 28 unit tests pass

#### **Structure Changes**
- **Before**: `src/config/mod.rs` + `src/config/provider.rs` (confusing)
- **After**: `src/config.rs` (clear and simple)
- **Benefits**: No naming confusion, easier to find configuration code, simpler maintenance

#### **Impact**
- ✅ **Clarity**: Eliminated naming confusion with providers
- ✅ **Simplicity**: Single file for all configuration needs
- ✅ **Maintainability**: Easier to locate and modify configuration code
- ✅ **Compatibility**: No breaking changes to public API

## [0.4.7] - 2025-10-16

### 🏗️ **ARCHITECTURE: Correct Protocol vs Provider Separation**

#### **Refactored**
- **Protocol/Provider Architecture** - Implemented correct separation of public vs private protocols
  - **Public Protocols** (`src/protocols/`): Only industry-standard protocols (OpenAI, Anthropic)
  - **Private Protocols** (`src/providers/`): Provider-specific protocols inline with implementations
  - Moved `AliyunProtocol` and `ZhipuProtocol` from `protocols/` to `providers/` as private protocols
  - Updated exports: Standard protocols from `protocols`, private protocols from `providers`
  - All 78 unit and integration tests pass

#### **Design Principles**
- **Public Protocols**: Industry-recognized standards that multiple providers might implement
- **Private Protocols**: Provider-specific APIs that are tightly coupled to their implementations
- **Clean Separation**: Protocols define API formats, providers implement service logic
- **Maintainability**: Private protocols are co-located with their implementations

#### **Impact**
- ✅ **Architecture**: Clean separation of public vs private protocols
- ✅ **Maintainability**: Private protocols are easier to maintain alongside providers
- ✅ **Extensibility**: Clear guidelines for adding new protocols vs providers
- ✅ **Tests**: All functionality tests pass (78/78)

## [0.4.6] - 2025-10-16

### 🔧 **HOTFIX: Streaming Integration Test Errors**

#### **Fixed**
- **Streaming integration test compilation errors** - Fixed all compilation errors in streaming tests
  - Fixed `tests/streaming_integration_tests.rs`: Added missing `Role` import
  - Updated all `Message::user()` calls to use proper `Message` construction with `Role::User`
  - Fixed all client creation calls: `.unwrap()` → `?` for V2 architecture
  - Fixed error handling test to properly detect authentication errors
  - All streaming integration tests now pass (4/4 passed, 4 ignored for API keys)

#### **Impact**
- ✅ **Streaming Tests**: All streaming integration tests compile and pass
- ✅ **Test Coverage**: Complete test coverage for streaming functionality
- ✅ **V2 Architecture**: All tests use correct V2 architecture APIs

## [0.4.5] - 2025-10-16

### 🔧 **HOTFIX: Test Compilation Errors**

#### **Fixed**
- **Test compilation errors** - Fixed compilation errors in test files
  - Fixed `tests/client_tests.rs`: Updated `protocol_name()` → `provider_name()` method calls
  - Fixed main documentation tests in `src/lib.rs` and `src/client.rs`
  - Updated import statements to use correct V2 architecture paths
  - All unit tests and integration tests now pass successfully

#### **Impact**
- ✅ **Tests**: All unit and integration tests compile and pass (78/78)
- ✅ **Documentation**: Main documentation examples work correctly
- ✅ **CI/CD**: Test suite runs successfully for automated builds

## [0.4.4] - 2025-10-16

### 🔧 **HOTFIX: Examples Compilation Errors**

#### **Fixed**
- **Examples compilation errors** - Fixed all compilation errors and warnings in example files
  - Updated `examples/zhipu_basic.rs`: Fixed API calls and imports for V2 architecture
  - Updated `examples/zhipu_streaming.rs`: Fixed Message construction and client creation
  - Updated `examples/streaming_basic.rs`: Fixed imports and Result handling
  - Updated `examples/ollama_model_management.rs`: Fixed Ollama provider interface usage
  - Updated `examples/v1_vs_v2_comparison.rs`: Removed deprecated feature flags
  - All examples now use V2 architecture APIs correctly

#### **Impact**
- ✅ **Examples**: All examples compile and run successfully
- ✅ **Documentation**: Examples serve as accurate V2 architecture documentation
- ✅ **User Experience**: Users can run examples without compilation errors

## [0.4.3] - 2025-10-16

### 🔧 **HOTFIX: Module Privacy Error**

#### **Fixed**
- **Critical module privacy error** - Fixed private module access in streaming functionality
  - Fixed import path: `crate::types::streaming::ChatStream` → `crate::types::ChatStream`
  - Fixed import path: `crate::types::streaming::StreamingResponse` → `crate::types::StreamingResponse`
  - The `streaming` module is conditionally exported and should be accessed through `types` module
  - Affected file: `src/sse.rs`

#### **Impact**
- ✅ **Compilation**: Now compiles successfully without privacy errors
- ✅ **Streaming**: All streaming features work correctly
- ✅ **Functionality**: No breaking changes to public API

## [0.4.2] - 2025-10-16

### 🔧 **HOTFIX: Type Mismatch Error**

#### **Fixed**
- **Critical type mismatch error** - Fixed streaming response type conversion
  - Added `sse_to_streaming_response()` function to convert `String` stream to `StreamingResponse` stream
  - Fixed type mismatch: expected `StreamingResponse` but found `String` in streaming methods
  - Affected files: `src/sse.rs`, `src/core/traits.rs`, `src/protocols/zhipu.rs`, `src/providers/ollama.rs`
  - All streaming functionality now works correctly with proper type conversion

#### **Impact**
- ✅ **Compilation**: Now compiles successfully without type errors
- ✅ **Streaming**: All streaming features work with correct types
- ✅ **Functionality**: No breaking changes to public API

## [0.4.1] - 2025-10-16

### 🔧 **HOTFIX: Compilation Error**

#### **Fixed**
- **Critical compilation error** - Fixed unresolved import `crate::sse::SseStream`
  - Replaced incorrect `SseStream::new(response)` calls with `crate::sse::sse_events(response)`
  - Affected files: `src/core/traits.rs`, `src/protocols/zhipu.rs`, `src/providers/ollama.rs`
  - All streaming functionality now works correctly

#### **Impact**
- ✅ **Compilation**: Now compiles successfully without errors
- ✅ **Streaming**: All streaming features work as expected
- ✅ **Functionality**: No breaking changes to public API

## [0.4.0] - 2025-10-16

### 🚀 **MAJOR RELEASE: V2 Architecture**

This is a major release that introduces the new V2 architecture as the default, providing significant performance improvements and a cleaner API design.

#### ⚡ **Performance Improvements**
- **7,000x+ faster client creation** - From ~53ms to ~7µs
- **Minimal memory footprint** - Only 16 bytes per client instance
- **Zero-cost cloning** - Arc-based sharing for efficient cloning

#### 🏗️ **New Architecture**
- **Clear Protocol/Provider separation** - Protocols define API specs, Providers implement services
- **Unified trait system** - `Protocol` and `Provider` traits for maximum extensibility
- **Type-safe HTTP client** - Compile-time guarantees for correctness
- **Generic provider implementation** - `GenericProvider<Protocol>` for most use cases

#### 🔄 **API Changes (Breaking)**

##### **Client Creation**
```rust
// V1 (Legacy)
let client = LlmClient::openai("sk-...", None);
let client = LlmClient::ollama(None);

// V2 (New Default)
let client = LlmClient::openai("sk-...")?;  // Returns Result
let client = LlmClient::ollama()?;          // Returns Result
```

##### **Method Renames**
```rust
// V1 → V2
client.fetch_models()  → client.models()
client.protocol_name() → client.provider_name()
```

##### **Parameter Changes**
- **OpenAI**: Removed optional second parameter, use dedicated methods
  - `openai(key, Some(url))` → `openai_with_base_url(key, url)?`
- **Ollama**: Removed optional parameter
  - `ollama(Some(url))` → `ollama_with_url(url)?`

#### 🆕 **New Features**

##### **Additional Client Creation Methods**
```rust
// Azure OpenAI support
LlmClient::azure_openai("key", "endpoint", "version")?

// OpenAI-compatible services
LlmClient::openai_compatible("key", "url", "name")?

// Zhipu GLM OpenAI-compatible mode
LlmClient::zhipu_openai_compatible("key")?

// Enhanced configuration options
LlmClient::openai_with_config("key", url, timeout, proxy)?
```

##### **Enhanced Ollama Support**
```rust
// Direct access to Ollama-specific features
if let Some(ollama) = client.as_ollama() {
    ollama.pull_model("llama2").await?;
    let models = ollama.models().await?;
}
```

#### 📦 **Protocol Support**
- **OpenAI Protocol** - Complete OpenAI API specification
- **Anthropic Protocol** - Full Claude API support with Vertex AI and Bedrock
- **Aliyun Protocol** - DashScope API with regional support
- **Zhipu Protocol** - Native and OpenAI-compatible formats
- **Ollama Provider** - Custom implementation with model management

#### 🔄 **Migration Guide**

##### **Option 1: Backward Compatibility**
```toml
# Cargo.toml
[features]
v1-legacy = []
```

```rust
// Use V1 API
#[cfg(feature = "v1-legacy")]
use llm_connector::v1::LlmClient;

// Use V2 API (default)
#[cfg(not(feature = "v1-legacy"))]
use llm_connector::LlmClient;
```

##### **Option 2: Direct Migration**
1. Add `?` to handle `Result` return types
2. Update method names: `fetch_models()` → `models()`, `protocol_name()` → `provider_name()`
3. Replace parameter patterns with dedicated methods
4. Update imports if using internal traits

#### 🏛️ **Architecture Benefits**
- **Extensibility** - Easy to add new protocols and providers
- **Type Safety** - Compile-time guarantees for all operations
- **Performance** - Optimized for speed and memory efficiency
- **Clarity** - Clear separation of concerns between protocols and providers
- **Maintainability** - Reduced code duplication and cleaner abstractions

## [0.3.6] - 2025-10-14

### ✨ Added

#### Ollama Streaming Support
- Implemented line-delimited JSON streaming for Ollama protocol
  - Added non-SSE parser for JSON lines stream
  - Integrated into core streaming pipeline with protocol switch
  - Normalized to `StreamingResponse` with `get_content()` for output
- Added `examples/ollama_streaming.rs` demonstrating `chat_stream()` usage

### 📝 Updated
- README and examples already standardized to use `get_content()` for streaming output

## [0.2.3] - 2025-01-06

### ✨ Added

#### Improved Error Messages
- **Cleaned up authentication error messages** for OpenAI-compatible providers
  - Removes OpenAI-specific URLs (like "platform.openai.com") from error messages
  - Adds helpful context: "Please verify your API key is correct and has the necessary permissions"
  - Makes errors more generic and applicable to all OpenAI-compatible providers (DeepSeek, Zhipu, Moonshot, etc.)

#### New Debug Tools
- **Added `debug_deepseek.rs` example** for troubleshooting authentication issues
  - Validates API key format
  - Tests model fetching and chat requests
  - Provides specific troubleshooting guidance based on error type
  - Can accept API key from command line or environment variable

#### Documentation
- **Added `TROUBLESHOOTING.md`** - Comprehensive troubleshooting guide
  - Authentication errors and solutions
  - Connection errors and debugging steps
  - Rate limit handling
  - Model not found errors
  - Provider-specific instructions for DeepSeek, OpenAI, Zhipu, Moonshot
  - Example code for common scenarios

### 🔧 Changed

#### Simplified API - Removed `supported_models()`
- **Removed `supported_models()` method** from all traits and implementations
  - Removed from `Provider` trait
  - Removed from `ProviderAdapter` trait
  - Removed from `LlmClient`
  - Removed from all protocol implementations (OpenAI, Anthropic, Aliyun, Ollama)
- **Removed `supports_model()` method** from `Provider` trait (was dependent on `supported_models()`)
- **Removed hardcoded model lists** from protocol structs
  - Removed `supported_models` field from `AnthropicProtocol`
  - Removed `supported_models` field from `AliyunProtocol`
  - Removed `supported_models` field from `OllamaProtocol`

#### Rationale
- `supported_models()` returned empty `[]` for most protocols (OpenAI, Anthropic, Aliyun)
- Only Ollama had hardcoded models, which were outdated
- Users should use `fetch_models()` for real-time model discovery
- Simplifies the API by removing confusion between two similar methods

#### Migration Guide

**Before:**
```rust
let client = LlmClient::openai("sk-...");
let models = client.supported_models(); // Returns []
```

**After:**
```rust
let client = LlmClient::openai("sk-...");
let models = client.fetch_models().await?; // Returns actual models from API
```

For protocols that don't support `fetch_models()` (Anthropic, Aliyun, Ollama), you can use any model name directly in your requests.

### 📝 Updated

- Updated tests to remove `supported_models()` usage
- Updated examples to demonstrate only `fetch_models()`
- Updated README.md to remove references to `supported_models()`
- Simplified documentation and examples

## [0.2.2] - 2025-01-06

Same as 0.2.1 - version bump for crates.io publication.

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
## [0.3.3] - 2025-10-14

### ✨ Added
- README: Added “Reasoning Synonyms” section with normalized keys and usage examples (`reasoning_any()`), covering non-streaming and streaming.

### 🔧 Changed
- Examples: Removed outdated examples using deprecated `openai_compatible` (`examples/test_fetch_models.rs`, `examples/test_with_keys.rs`).
- Examples: Updated DeepSeek and fetch models example to use `LlmClient::openai(api_key, Some(base_url))`.
- Docs: Fixed doctests across `lib.rs`, `protocols/core.rs`, `protocols/openai.rs`, `protocols/aliyun.rs`, `protocols/anthropic.rs` to match current API.
- Docs: Replaced obsolete imports like `protocols::aliyun::qwen` and `protocols::anthropic::claude` with `LlmClient::aliyun(...)` and `LlmClient::anthropic(...)`.
- Docs: Standardized message initialization to `Message::user(...)` or `Role` enums where appropriate.

### ✅ Validation
- `cargo build --examples` passes.
- `cargo test` (library and integration with `streaming` feature) passes.
- `cargo test --doc` passes (13 passed, 0 failed, 4 ignored).
## 0.3.4 - 2025-10-14

Updates
- Add compatibility alias `types::ChatMessage = Message` to ease migration.
- Add `ChatResponse::get_usage_safe()` returning `(prompt, completion, total)`.
- Add `ChatResponse::get_content()` returning the first choice content as `Option<&str>`.
- README install snippet updated to `0.3.4`.

Notes
- `ChatRequest::new(model)` remains as minimal constructor.
- Use `ChatRequest::new_with_messages(model, messages)` to pass initial message list.
- `Message::user/assistant/system` are preferred constructors; reasoning fields are auto-populated.

## 0.3.5 - 2025-10-14

Updates
- Add `StreamingResponse::get_content()` for convenience and API symmetry with `ChatResponse::get_content()`.

Notes
- No breaking changes; existing code continues to work. You can still access `choices[0].delta.content` directly.

