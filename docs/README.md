# llm-connector 文档索引

## 📚 核心文档

### 架构和设计
- [ARCHITECTURE.md](ARCHITECTURE.md) - 项目架构文档
- [MULTIMODAL_NATIVE_DESIGN.md](MULTIMODAL_NATIVE_DESIGN.md) - 多模态内容设计

### 使用指南
- [MIGRATION_GUIDE_v0.5.0.md](MIGRATION_GUIDE_v0.5.0.md) - v0.5.0 迁移指南
- [REASONING_MODELS_SUPPORT.md](REASONING_MODELS_SUPPORT.md) - 推理模型通用支持指南

### 开发规范
- [RUST_PROJECT_GUIDELINES.md](RUST_PROJECT_GUIDELINES.md) - Rust 项目开发规范

## 🔌 Provider 使用指南

所有 Provider 的详细使用文档都在 `guides/` 目录下：

- [guides/ALIYUN_GUIDE.md](guides/ALIYUN_GUIDE.md) - 阿里云 DashScope 使用指南
- [guides/ANTHROPIC_GUIDE.md](guides/ANTHROPIC_GUIDE.md) - Anthropic Claude 使用指南
- [guides/DEEPSEEK_GUIDE.md](guides/DEEPSEEK_GUIDE.md) - DeepSeek 使用指南
- [guides/MOONSHOT_GUIDE.md](guides/MOONSHOT_GUIDE.md) - Moonshot 使用指南
- [guides/TENCENT_GUIDE.md](guides/TENCENT_GUIDE.md) - 腾讯混元使用指南
- [guides/VOLCENGINE_GUIDE.md](guides/VOLCENGINE_GUIDE.md) - 火山引擎使用指南
- [guides/ZHIPU_GUIDE.md](guides/ZHIPU_GUIDE.md) - 智谱 GLM 使用指南

## 📦 归档文档

历史版本发布说明和测试报告已移至 `archive/` 目录：

- `archive/releases/` - 历史版本发布说明
- `archive/reports/` - 历史测试报告和重构总结

## 🚀 快速开始

1. **新用户**: 从主 [README.md](../README.md) 开始
2. **迁移用户**: 查看 [MIGRATION_GUIDE_v0.5.0.md](MIGRATION_GUIDE_v0.5.0.md)
3. **使用推理模型**: 查看 [REASONING_MODELS_SUPPORT.md](REASONING_MODELS_SUPPORT.md)
4. **特定 Provider**: 查看 `guides/` 目录下对应的指南

## 📝 文档维护

### 文档结构

```
docs/
├── README.md                           # 本文档
├── ARCHITECTURE.md                     # 架构文档
├── MULTIMODAL_NATIVE_DESIGN.md        # 多模态设计
├── MIGRATION_GUIDE_v0.5.0.md          # 迁移指南
├── REASONING_MODELS_SUPPORT.md        # 推理模型支持
├── RUST_PROJECT_GUIDELINES.md         # Rust 规范
├── guides/                             # Provider 使用指南
│   ├── ALIYUN_GUIDE.md
│   ├── ANTHROPIC_GUIDE.md
│   ├── DEEPSEEK_GUIDE.md
│   ├── MOONSHOT_GUIDE.md
│   ├── TENCENT_GUIDE.md
│   ├── VOLCENGINE_GUIDE.md
│   └── ZHIPU_GUIDE.md
└── archive/                            # 归档文档
    ├── releases/                       # 历史版本发布说明
    └── reports/                        # 历史测试报告
```

### 文档更新原则

1. **核心文档**: 保持最新，反映当前版本的功能和设计
2. **Provider 指南**: 每个 Provider 一个独立文档，包含基础用法、特殊功能、常见问题
3. **归档文档**: 历史文档移至 archive，保留但不再更新

## 🔗 相关链接

- [项目主页](https://github.com/lipish/llm-connector)
- [API 文档](https://docs.rs/llm-connector)
- [Crates.io](https://crates.io/crates/llm-connector)
- [更新日志](../CHANGELOG.md)

## 📧 反馈

如果发现文档问题或有改进建议，请：
1. 提交 Issue: https://github.com/lipish/llm-connector/issues
2. 提交 PR: https://github.com/lipish/llm-connector/pulls

