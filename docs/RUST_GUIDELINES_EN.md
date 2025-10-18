# Rust Project Development Guidelines

## 1. Design Philosophy

### 1.1 Minimalist Design Principles
- **Minimal public surface**: Expose the smallest necessary API surface; avoid over-engineering.
- **No hard-coded constraints**: Avoid hard-coding configuration, model names, or provider details in code.
- **Avoid complex configuration**: Minimize unnecessary configuration files and registries; offer sensible defaults.
- **Clear abstractions**: Provide intuitive, direct API usage and straightforward abstraction layers.

### 1.2 Protocols / Interfaces First
- **Group by functionality**: Organize code by API protocol or functional domain rather than by company/product.
- **Shared implementations**: Reuse implementation logic for similar features to avoid duplication.
- **Extensible architecture**: Design extensibility through interfaces and adapter patterns.

### 1.3 Type Safety and Error Handling
- **Strong typing**: Leverage Rust's type system to reduce runtime errors.
- **Unified error types**: Define dedicated error types to provide a consistent error handling experience.
- **Rich error context**: Errors should include sufficient contextual information to aid debugging.
- **Reasonable propagation**: Use the `?` operator to simplify error propagation where appropriate.

## 2. Code Organization

### 2.1 Module Design
- **Separation of concerns**: Organize code into modules by feature and abstraction level.
- **Flat structure**: Avoid deep nesting; prefer a flatter modular hierarchy.
- **Core vs extensions**: Clearly separate core functionality from optional/extension features.
- **Protocol / adapter patterns**: Use protocol and adapter patterns to support multiple backends.
- **Documentation location**: Except for the `README`, place all documentation in the `docs/` directory.
- **Tests location**: Put all tests in the `tests/` directory.

### 2.2 Recommended Project Layout
/
  /docs             # Documentation
    /api            # API documentation
    /guides         # Guides
    /tutorials      # Tutorials
  /src              # Source code
    /client         # Client interfaces
    /config         # Configuration types and implementations
    /protocols      # Protocol abstractions and implementations
      /core         # Core abstraction interfaces
      /impl1        # Concrete implementation 1
      /impl2        # Concrete implementation 2
    /types          # Shared data types
    /error          # Error handling
    /utils          # Utility functions
    /features       # Optional feature modules
    lib.rs          # Library entry point, export public API
    main.rs         # Application entry point (if applicable)
  /tests            # Integration tests
    /common         # Test shared code
    client_tests.rs # Client tests
    protocol_tests.rs # Protocol tests
  README.md         # Single top-level README
  CHANGELOG.md      # Changelog
  Cargo.toml        # Rust package manifest

### 2.3 Directory Creation Rules
- **`docs` directory**: Projects must include a `docs/` directory for all documentation (except README).
- **`tests` directory**: Projects must include a `tests/` directory for integration tests.
- **Auto-create**: Initialize new projects with these directories created automatically.
- **Clear purpose**: Every directory should have a clearly stated purpose and organizational guidelines.

## 3. Naming Conventions

### 3.1 Basic Naming Rules
- **Structs and enums**: CamelCase (e.g., `ChatRequest`, `LlmClient`).
- **Functions and methods**: snake_case (e.g., `fetch_models`, `chat_stream`).
- **Modules and files**: snake_case (e.g., `error.rs`, `http_transport.rs`).
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRY_ATTEMPTS`).
- **Traits**: CamelCase, often with `-able`/`Adapter` suffixes (e.g., `ProviderAdapter`, `ErrorMapper`).

### 3.2 Special Naming Conventions
- **Protocol types**: Use `Protocol` suffix with the protocol name (e.g., `OpenAIProtocol`).
- **Internal/private members**: Use underscore prefix for intentionally private/internal fields (e.g., `_internal_field`).
- **Type parameters**: Single uppercase letters (e.g., `T`, `U`, `R`).
- **Lifetimes**: Single-quoted lowercase letters (e.g., `'a`, `'static`).

## 4. Code Style and Guidelines

### 4.1 Documentation Guidelines
- **Module-level docs**: Each module should include detailed documentation comments explaining its purpose and usage.
- **Public API docs**: All public functions, structs, enums, etc., should have comprehensive doc comments.
- **Examples**: Provide concise usage examples demonstrating typical API patterns.
- **Doc tests**: Use code examples in `///` comments as doctests.

### 4.2 Formatting
- **Use rustfmt**: Follow rustfmt default formatting.
- **Line length**: Prefer to keep lines within 100 characters when possible.
- **Whitespace**: Use blank lines appropriately to improve readability.
- **Brace style**: Follow Rust standard library style (opening brace on the same line).

### 4.3 Comment Best Practices
- **Explain the "why"**: Comments should explain design decisions and non-obvious logic.
- **Avoid redundant comments**: Do not comment what is already obvious from the code.
- **TODO comments**: Use `TODO:` to mark pending work, and include owner or deadline if possible.
- **FIXME comments**: Use `FIXME:` to mark known issues or areas for improvement.

## 5. Abstraction and Design Patterns

### 5.1 Core Abstractions
- **Provider pattern**: Define a unified `Provider` interface to encapsulate different backend implementations.
- **Adapter pattern**: Use adapters to translate between varying API request/response formats.
- **ErrorMapper**: Use a dedicated component for error conversion and mapping.

### 5.2 Idiomatic Rust Patterns
- **Zero-cost abstractions**: Favor abstractions that don't introduce runtime overhead.
- **Resource management**: Use RAII for resource management.
- **Ownership semantics**: Use ownership, borrowing, and lifetimes appropriately.
- **Generics**: Use generics for reusable algorithms and data structures.

### 5.3 Concurrency and Async Patterns
- **Async-first**: Prefer async implementations for I/O-bound operations.
- **Zero-copy sharing**: Use `Arc` for efficient shared ownership where appropriate.
- **Thread safety**: Ensure shared data is thread-safe.
- **Cancellation support**: Provide cancellation mechanisms for async operations.

## 6. Dependency Management

### 6.1 Dependency Selection Principles
- **Minimal dependencies**: Choose only necessary dependencies to avoid dependency bloat.
- **Actively maintained**: Prefer well-maintained libraries.
- **Version locking**: Rely on `Cargo.lock` to lock versions for reproducible builds.

### 6.2 Feature Flags
- **Optional functionality**: Use Cargo feature flags to gate optional features.
- **Composable features**: Design feature flags to be composable.
- **Sensible defaults**: Enable core functionality by default; optional features should be opt-in.

### 6.3 Dev Dependencies
- **Test dependencies**: Place test-only deps under `[dev-dependencies]`.
- **Tooling deps**: Manage development tools (clippy, rustfmt configurations) separately.

## 7. API Design Principles

### 7.1 Consistent Interfaces
- **Consistent API style**: Keep public API style consistent across the project.
- **Simple constructors**: Offer convenient static constructor functions.
- **Reasonable defaults**: Provide sensible defaults for optional parameters.
- **Builder pattern**: Use builder patterns for complex configurations.

### 7.2 Type-safe Interfaces
- **Domain-specific types**: Create dedicated types for domain concepts to avoid misuse of primitive types.
- **Enums over booleans**: Prefer enums to boolean flags for clearer semantics.
- **Compile-time checks**: Move errors to compile-time where possible.
- **Type aliases**: Use aliases to simplify complex type signatures.

### 7.3 Compatibility Considerations
- **Backward compatibility**: Breaking changes to public APIs should be avoided where possible.
- **Semantic versioning**: Follow SemVer for releases.
- **Migration guides**: Provide migration instructions for API changes.

## 8. Testing and Quality Assurance

### 8.1 Testing Strategy
- **Unit tests**: Write unit tests for every public function and module.
- **Integration tests**: Test interactions between components.
- **Doc tests**: Ensure example code in docs runs correctly.
- **Edge case tests**: Cover boundary conditions and error scenarios.

### 8.2 Quality Tools
- **Clippy**: Enable and follow Clippy lints.
- **rustfmt**: Automatically format code.
- **Static analysis**: Consider additional static analysis tools when appropriate.
- **Performance profiling**: Profile performance-critical paths.

## 9. Anti-patterns (What to Avoid)

### 9.1 Design Anti-patterns
- **Over-engineering**: Avoid unnecessary complex abstractions and unneeded features.
- **Premature optimization**: Prefer clear, correct code first, optimize later when necessary.
- **Hard-coded values**: Avoid embedding config or environment-specific values in code.
- **Violating single responsibility**: Ensure each component has a single responsibility.

### 9.2 Rust-specific Anti-patterns
- **Unnecessary cloning**: Avoid needless clones of data.
- **Overuse of unwrap**: Avoid using `unwrap()` and `expect()` for recoverable errors.
- **Ignoring ownership**: Do not ignore or misunderstand Rust ownership semantics.
- **Excessive unsafe**: Avoid `unsafe` unless absolutely necessary and justified.

## 10. Project Documentation

### 10.1 README
- **Project overview**: Explain the project purpose and main features clearly.
- **Quick start**: Provide a concise getting-started guide.
- **Examples**: Show typical use cases with example code.
- **Installation**: Detail installation and configuration steps.

### 10.2 CHANGELOG
- **Version history**: Record changes per release.
- **Change types**: Differentiate features, fixes, and optimizations.
- **Migration notes**: Offer migration steps when upgrading versions.

### 10.3 CONTRIBUTING Guide
- **Developer setup**: Describe how to set up a local development environment.
- **Code conventions**: State project-specific coding standards and best practices.
- **Commit conventions**: Define preferred Git commit message format and content.
- **PR workflow**: Document the pull request submission and review process.

---

These guidelines are based on Rust design patterns (see https://rust-unofficial.github.io/patterns/) and best practices from the llm-connector project. They are intended as a comprehensive reference for Rust project development and may be adapted to suit specific project needs.