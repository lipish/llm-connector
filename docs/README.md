# llm-connector Documentation Index

## Core Documentation

### Architecture and Design
- [ARCHITECTURE.md](ARCHITECTURE.md) - Project architecture documentation
- [MULTIMODAL_NATIVE_DESIGN.md](MULTIMODAL_NATIVE_DESIGN.md) - Multi-modal content design

### User Guides
- [MIGRATION_GUIDE_v0.5.0.md](MIGRATION_GUIDE_v0.5.0.md) - v0.5.0 migration guide
- [REASONING_MODELS_SUPPORT.md](REASONING_MODELS_SUPPORT.md) - Universal reasoning models support guide
- [STREAMING_TOOL_CALLS.md](STREAMING_TOOL_CALLS.md) - Streaming tool calls support documentation

### Technical Documentation
- [STREAMING_TOOL_CALLS_FIX.md](STREAMING_TOOL_CALLS_FIX.md) - Streaming tool calls fix summary

### Development Guidelines
- [RUST_PROJECT_GUIDELINES.md](RUST_PROJECT_GUIDELINES.md) - Rust project development guidelines

## Provider Usage Guides

Detailed usage documentation for all providers is in the `guides/` directory:

- [guides/ALIYUN_GUIDE.md](guides/ALIYUN_GUIDE.md) - Aliyun DashScope usage guide
- [guides/ANTHROPIC_GUIDE.md](guides/ANTHROPIC_GUIDE.md) - Anthropic Claude usage guide
- [guides/DEEPSEEK_GUIDE.md](guides/DEEPSEEK_GUIDE.md) - DeepSeek usage guide
- [guides/MOONSHOT_GUIDE.md](guides/MOONSHOT_GUIDE.md) - Moonshot usage guide
- [guides/TENCENT_GUIDE.md](guides/TENCENT_GUIDE.md) - Tencent Hunyuan usage guide
- [guides/VOLCENGINE_GUIDE.md](guides/VOLCENGINE_GUIDE.md) - Volcengine usage guide
- [guides/ZHIPU_GUIDE.md](guides/ZHIPU_GUIDE.md) - Zhipu GLM usage guide

## Archived Documentation

Historical release notes and test reports have been moved to the `archive/` directory:

- `archive/releases/` - Historical release notes
- `archive/reports/` - Historical test reports and refactoring summaries

## Quick Start

1. **New Users**: Start with the main [README.md](../README.md)
2. **Migrating Users**: See [MIGRATION_GUIDE_v0.5.0.md](MIGRATION_GUIDE_v0.5.0.md)
3. **Using Reasoning Models**: See [REASONING_MODELS_SUPPORT.md](REASONING_MODELS_SUPPORT.md)
4. **Specific Provider**: See the corresponding guide in the `guides/` directory

## Documentation Maintenance

### Documentation Structure

```
docs/
├── README.md                           # This document
├── ARCHITECTURE.md                     # Architecture documentation
├── MULTIMODAL_NATIVE_DESIGN.md        # Multi-modal design
├── MIGRATION_GUIDE_v0.5.0.md          # Migration guide
├── REASONING_MODELS_SUPPORT.md        # Reasoning models support
├── STREAMING_TOOL_CALLS.md            # Streaming tool calls support
├── STREAMING_TOOL_CALLS_FIX.md        # Streaming tool calls fix summary
├── RUST_PROJECT_GUIDELINES.md         # Rust guidelines
├── guides/                             # Provider usage guides
│   ├── ALIYUN_GUIDE.md
│   ├── ANTHROPIC_GUIDE.md
│   ├── DEEPSEEK_GUIDE.md
│   ├── MOONSHOT_GUIDE.md
│   ├── TENCENT_GUIDE.md
│   ├── VOLCENGINE_GUIDE.md
│   └── ZHIPU_GUIDE.md
└── archive/                            # Archived documentation
    ├── releases/                       # Historical release notes
    └── reports/                        # Historical test reports
```

### Documentation Update Principles

1. **Core Documentation**: Keep up-to-date, reflecting current version features and design
2. **Provider Guides**: One document per provider, including basic usage, special features, and common issues
3. **Archived Documentation**: Historical documents moved to archive, preserved but no longer updated

## Related Links

- [Project Homepage](https://github.com/lipish/llm-connector)
- [API Documentation](https://docs.rs/llm-connector)
- [Crates.io](https://crates.io/crates/llm-connector)
- [Changelog](../CHANGELOG.md)

## Feedback

If you find documentation issues or have improvement suggestions:
1. Submit an Issue: https://github.com/lipish/llm-connector/issues
2. Submit a PR: https://github.com/lipish/llm-connector/pulls

