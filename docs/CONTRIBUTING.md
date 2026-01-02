# Contributing to llm-connector

We welcome contributions! Please follow these guidelines to ensure code quality and consistency.

## Rust Coding Guidelines

See [RUST_PROJECT_GUIDELINES.md](RUST_PROJECT_GUIDELINES.md) (Note: This content is now consolidated here).

### General Rules
1.  **Zero Warnings**: The codebase must compile with 0 warnings.
2.  **No Unused Code**: Clean up dead code, redundant imports, and unused variables.
3.  **English Only**: correct, professional English for all comments, documentation, and commit messages. No Chinese characters in source code.

### Code Style
- Use `rustfmt` for formatting.
- Follow idioms such as `Builder` pattern for complex object creation.
- Prefer `Option<&str>` over `Option<String>` for arguments when possible.

### Error Handling
- Use `LlmConnectorError` for all library errors.
- Never use `unwrap()` in library code; use `?` or `expect()` with a descriptive message if panic is absolutely necessary/invariant.

## Security

### Sensitive Information
- **NEVER** commit real API keys or secrets.
- Use placeholders like `sk-...` or `your-api-key` in examples and documentation.
- Use environment variables (e.g., `OPENAI_API_KEY`) for tests.

## Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run tests for a specific feature
cargo test --features tencent
```

### Writing Tests
- **Integration Tests**: Place in `examples/` or `tests/`.
- **Unit Tests**: Place in `mod tests` within the source file.
- **Mocking**: Use `wiremock` for network tests to avoid hitting real APIs during CI.
