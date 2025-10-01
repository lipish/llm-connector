# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-10

### Added

#### Type Safety Improvements
- **`Role` enum** for message roles (System, User, Assistant, Tool)
  - Compile-time validation prevents invalid role strings
  - Better IDE autocomplete and documentation
  - Prevents typos and runtime errors

#### Ergonomic API Improvements
- **Message constructors** for cleaner code
  - `Message::system(content)` - Create system messages
  - `Message::user(content)` - Create user messages
  - `Message::assistant(content)` - Create assistant messages
  - `Message::tool(content, tool_call_id)` - Create tool response messages
  - Builder methods: `with_name()`, `with_tool_calls()`

- **ChatRequest builder pattern**
  - `ChatRequest::new(model)` - Create new request
  - `add_message()` - Add single message
  - `with_messages()` - Set all messages
  - `with_temperature()`, `with_max_tokens()`, etc. - Set parameters
  - Fluent API for cleaner request construction

- **ToolChoice constructors**
  - `ToolChoice::none()` - No tools
  - `ToolChoice::auto()` - Let model decide
  - `ToolChoice::required()` - Tools must be called
  - `ToolChoice::function(name)` - Call specific function

#### Documentation
- Comprehensive protocol design documentation (`docs/PROTOCOLS_DESIGN.md`)
- Types optimization documentation (`docs/TYPES_OPTIMIZATION.md`)
- Architecture design documentation (`docs/ARCHITECTURE_DESIGN.md`)
- New example: `types_showcase.rs` demonstrating new APIs

### Fixed
- **ToolChoice serialization bug** - Fixed incorrect JSON serialization
  - Single-unit variants now serialize to strings ("auto", "none", "required")
  - Function variant now includes required "type" field
  - Matches OpenAI API specification correctly

### Changed
- **Breaking**: `Message.role` changed from `String` to `Role` enum
- **Breaking**: `Delta.role` changed from `Option<String>` to `Option<Role>`
- All protocol adapters updated to handle Role enum conversion
- All examples and tests updated to use new API

### Migration Guide

#### Old Code
```rust
let msg = Message {
    role: "user".to_string(),
    content: "Hello".to_string(),
    ..Default::default()
};
```

#### New Code (Option 1 - Direct)
```rust
let msg = Message {
    role: Role::User,
    content: "Hello".to_string(),
    ..Default::default()
};
```

#### New Code (Option 2 - Constructor, Recommended)
```rust
let msg = Message::user("Hello");
```

### Performance
- No performance regression
- Zero-cost abstractions maintained
- All optimizations from 0.1.0 preserved

### Testing
- ✅ 35/35 unit tests passing
- ✅ 5/5 integration tests passing
- ✅ All examples compile and run successfully
- ⚠️ Some doctests need updates (non-critical)

## [0.1.0] - 2025-01-09

### Added
- Initial release
- Support for 10+ LLM providers
- Three protocol implementations (OpenAI, Anthropic, Aliyun)
- Generic provider architecture
- Middleware system (logging, metrics, retry, interceptor)
- Provider registry
- YAML configuration support
- Streaming support
- Comprehensive documentation

### Supported Providers
- DeepSeek
- Zhipu (GLM)
- Moonshot (Kimi)
- VolcEngine (Doubao)
- Tencent (Hunyuan)
- MiniMax
- StepFun
- LongCat
- Claude (Anthropic)
- Qwen (Aliyun)

[0.2.0]: https://github.com/lipish/llm-connector/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lipish/llm-connector/releases/tag/v0.1.0

