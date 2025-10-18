# llm-connector 文档目录

欢迎查阅  项目文档！

## 📚 文档索引

### 核心文档

- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - V2 架构设计文档
  - Protocol/Provider 双层架构说明
  - 设计哲学与核心组件
  - 适用于理解项目整体设计

- **[DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md)** - 开发者指南
  - 如何贡献代码
  - 开发环境设置
  - 测试和 CI/CD 流程

- **[MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)** - 版本迁移指南
  - V1 到 V2 迁移步骤
  - Breaking Changes 说明
  - 向后兼容性信息

### 命名规范

- **[NAMING_CONVENTIONS.md](./NAMING_CONVENTIONS.md)** - llm-connector 项目命名规范 ⭐
  - 构造函数命名模式
  - 参数命名标准
  - Provider 类型转换规范
  - **推荐**: 所有贡献者必读

- **[UNIVERSAL_NAMING_GUIDE.md](./UNIVERSAL_NAMING_GUIDE.md)** - 通用 API 命名指南 ⭐⭐
  - 适用于任何 Rust 项目的命名最佳实践
  - 构造函数、方法、类型、模块命名模式
  - 12 章完整指南 + 速查表
  - **推荐**: 架构师和技术负责人必读

### Rust 编码规范

- **[RUST_PROJECT_CORE_RULES.md](./RUST_PROJECT_CORE_RULES.md)** - Rust 核心规则
  - 项目级 Rust 编码标准
  - 最佳实践和反模式

- **[RUST_PROJECT_GUIDELINES.md](./RUST_PROJECT_GUIDELINES.md)** - Rust 项目指南 (中文)
  - 详细的 Rust 编码规范
  - 错误处理、性能优化等主题

- **[RUST_PROJECT_GUIDELINES_EN.md](./RUST_PROJECT_GUIDELINES_EN.md)** - Rust Project Guidelines (English)
  - 英文版 Rust 编码规范

### 其他文档

- **[README_ZH.md](./README_ZH.md)** - 中文 README
  - 项目介绍和快速开始指南

---

## 🎯 快速导航

### 我想...

- **了解项目架构** → [ARCHITECTURE.md](./ARCHITECTURE.md)
- **开始贡献代码** → [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) + [NAMING_CONVENTIONS.md](./NAMING_CONVENTIONS.md)
- **从 V1 迁移到 V2** → [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)
- **学习 API 命名最佳实践** → [UNIVERSAL_NAMING_GUIDE.md](./UNIVERSAL_NAMING_GUIDE.md)
- **查看 Rust 编码规范** → [RUST_PROJECT_GUIDELINES.md](./RUST_PROJECT_GUIDELINES.md)

---

## 📖 阅读顺序建议

### 新贡献者
1. [ARCHITECTURE.md](./ARCHITECTURE.md) - 理解设计
2. [NAMING_CONVENTIONS.md](./NAMING_CONVENTIONS.md) - 学习命名规范
3. [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) - 开始贡献

### 项目维护者
1. [UNIVERSAL_NAMING_GUIDE.md](./UNIVERSAL_NAMING_GUIDE.md) - 通用命名原则
2. [ARCHITECTURE.md](./ARCHITECTURE.md) - 架构决策
3. [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) - 版本演进策略

### Rust 学习者
1. [RUST_PROJECT_CORE_RULES.md](./RUST_PROJECT_CORE_RULES.md) - 快速上手
2. [RUST_PROJECT_GUIDELINES.md](./RUST_PROJECT_GUIDELINES.md) - 深入学习
3. [UNIVERSAL_NAMING_GUIDE.md](./UNIVERSAL_NAMING_GUIDE.md) - 最佳实践

---

## 🔄 文档更新

所有文档遵循以下版本控制：

- **主版本号变更** - 架构重大变更时更新
- **次版本号变更** - 新增章节或大幅修订时更新
- **修订号变更** - 错误修正或小幅调整

当前文档版本: **v2.0** (对应 llm-connector v0.5.x)

---

**最后更新**: 2025-01-18  
**维护者**: llm-connector 核心团队
