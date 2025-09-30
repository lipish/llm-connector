# llm-connector 示例文档

本目录包含了 llm-connector 库的各种使用示例，帮助您快速上手并了解如何使用不同的功能。

## 📋 目录

- [快速开始](#快速开始)
- [环境变量配置](#环境变量配置)
- [示例列表](#示例列表)
- [运行示例](#运行示例)
- [常见问题](#常见问题)

---

## 🚀 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/lipish/llm-connector.git
cd llm-connector
```

### 2. 配置环境变量

根据您要使用的提供商，设置相应的 API 密钥：

```bash
# DeepSeek
export DEEPSEEK_API_KEY="your-deepseek-api-key"

# 阿里云 DashScope (通义千问)
export ALIYUN_API_KEY="your-aliyun-api-key"

# 智谱 AI (GLM)
export ZHIPU_API_KEY="your-zhipu-api-key"

# LongCat (免费额度)
export LONGCAT_API_KEY="your-longcat-api-key"

# Moonshot (Kimi)
export MOONSHOT_API_KEY="your-moonshot-api-key"
```

### 3. 运行示例

```bash
# 运行 DeepSeek 示例
cargo run --example deepseek_example

# 运行 LongCat 示例
cargo run --example longcat_demo
```

---

## 🔑 环境变量配置

### 支持的环境变量

| 环境变量 | 提供商 | 获取方式 | 是否必需 |
|---------|--------|---------|---------|
| `DEEPSEEK_API_KEY` | DeepSeek | [DeepSeek 平台](https://platform.deepseek.com/) | 可选 |
| `ALIYUN_API_KEY` | 阿里云 DashScope | [阿里云控制台](https://dashscope.console.aliyun.com/) | 可选 |
| `ZHIPU_API_KEY` | 智谱 AI | [智谱开放平台](https://open.bigmodel.cn/) | 可选 |
| `LONGCAT_API_KEY` | LongCat | [LongCat 平台](https://longcat.chat/platform/) | 可选 |
| `MOONSHOT_API_KEY` | Moonshot | [Moonshot 平台](https://platform.moonshot.cn/) | 可选 |

### 配置方法

#### 方法 1: 临时设置（当前终端会话）

```bash
export DEEPSEEK_API_KEY="sk-xxxxxxxxxxxxxxxx"
```

#### 方法 2: 永久设置（添加到 shell 配置文件）

**Bash (~/.bashrc 或 ~/.bash_profile):**
```bash
echo 'export DEEPSEEK_API_KEY="sk-xxxxxxxxxxxxxxxx"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh (~/.zshrc):**
```bash
echo 'export DEEPSEEK_API_KEY="sk-xxxxxxxxxxxxxxxx"' >> ~/.zshrc
source ~/.zshrc
```

#### 方法 3: 使用 .env 文件（推荐用于开发）

创建 `.env` 文件（注意：不要提交到 Git）：

```bash
# .env
DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxxxxx
ALIYUN_API_KEY=sk-xxxxxxxxxxxxxxxx
ZHIPU_API_KEY=xxxxxxxxxxxxxxxx
LONGCAT_API_KEY=ak_xxxxxxxxxxxxxxxx
MOONSHOT_API_KEY=sk-xxxxxxxxxxxxxxxx
```

然后使用 `direnv` 或在运行前手动加载：

```bash
# 手动加载
source .env

# 或使用 direnv (需要先安装)
direnv allow
```

---

## 📚 示例列表

### 1. `deepseek_example.rs` - DeepSeek 基础示例

**功能：** 演示如何使用 DeepSeek 提供商进行基本的聊天对话。

**特点：**
- 支持环境变量和手动配置两种方式
- 展示基本的请求/响应流程
- 包含错误处理示例

**运行：**
```bash
export DEEPSEEK_API_KEY="your-api-key"
cargo run --example deepseek_example
```

**适用场景：**
- 初次使用 llm-connector
- 学习基本的 API 调用流程
- 了解 DeepSeek 提供商的使用方法

---

### 2. `longcat_demo.rs` - LongCat 完整演示

**功能：** 演示 LongCat API 的完整使用流程，包括多种模型和配置选项。

**特点：**
- LongCat 提供免费的每日额度（500,000 tokens）
- 支持多种模型：LongCat-Flash-Chat、LongCat-Flash-Thinking
- 展示工厂模式的使用
- 包含详细的配置说明

**运行：**
```bash
export LONGCAT_API_KEY="your-api-key"
cargo run --example longcat_demo
```

**适用场景：**
- 想要使用免费的 LLM API
- 学习工厂模式的使用
- 了解 LongCat 平台的特性

**获取 API 密钥：**
1. 访问 [LongCat 平台](https://longcat.chat/platform/)
2. 注册账号
3. 在 API Keys 页面创建密钥
4. 免费额度：500,000 tokens/天（可申请增加到 5,000,000）

---

### 3. `protocol_architecture_demo.rs` - 协议架构演示

**功能：** 展示 llm-connector 的核心架构设计 - 基于协议的提供商组织方式。

**特点：**
- 演示三种协议：OpenAI、Anthropic、Aliyun
- 展示如何添加新的提供商
- 说明协议适配器的工作原理
- 不需要真实的 API 密钥（仅演示架构）

**运行：**
```bash
cargo run --example protocol_architecture_demo
```

**适用场景：**
- 了解项目的架构设计
- 学习如何扩展新的提供商
- 理解协议适配器模式

---

### 4. `test_all_providers.rs` - 测试所有提供商

**功能：** 批量测试所有配置的提供商，验证 API 连接和功能。

**特点：**
- 支持多个提供商：DeepSeek、Aliyun、Zhipu、LongCat、Moonshot
- 自动跳过未配置的提供商
- 展示每个提供商的响应结果
- 适合用于验证配置是否正确

**运行：**
```bash
# 设置所有需要测试的提供商的 API 密钥
export DEEPSEEK_API_KEY="your-deepseek-key"
export ALIYUN_API_KEY="your-aliyun-key"
export ZHIPU_API_KEY="your-zhipu-key"
export LONGCAT_API_KEY="your-longcat-key"
export MOONSHOT_API_KEY="your-moonshot-key"

cargo run --example test_all_providers
```

**适用场景：**
- 验证多个提供商的配置
- 批量测试 API 连接
- 比较不同提供商的响应

---

### 5. `verify_real_api_calls.rs` - 验证真实 API 调用

**功能：** 通过多次不同的请求验证是否在进行真实的 API 调用。

**特点：**
- 发送多个不同的问题
- 显示详细的请求和响应信息
- 验证响应的多样性
- 适合用于调试和验证

**运行：**
```bash
export DEEPSEEK_API_KEY="your-api-key"
cargo run --example verify_real_api_calls
```

**适用场景：**
- 验证 API 调用是否真实
- 调试请求/响应流程
- 测试不同的提示词效果

---

### 6. `providers.toml` - 配置文件示例

**功能：** 展示如何使用 TOML 配置文件管理多个提供商。

**特点：**
- 统一的配置格式
- 支持多个提供商
- 包含所有配置选项的说明
- 可以作为模板使用

**使用方法：**
```bash
# 复制示例配置
cp examples/providers.toml my-providers.toml

# 编辑配置文件，填入真实的 API 密钥
vim my-providers.toml

# 在代码中加载配置
# (需要启用 config 特性)
```

---

## 🎯 运行示例

### 基本运行

```bash
# 运行单个示例
cargo run --example <example_name>

# 例如：
cargo run --example deepseek_example
```

### 带环境变量运行

```bash
# 方法 1: 在命令前设置
DEEPSEEK_API_KEY="your-key" cargo run --example deepseek_example

# 方法 2: 先导出再运行
export DEEPSEEK_API_KEY="your-key"
cargo run --example deepseek_example
```

### 查看详细输出

```bash
# 启用日志输出
RUST_LOG=debug cargo run --example deepseek_example

# 或者使用 trace 级别
RUST_LOG=trace cargo run --example deepseek_example
```

### 编译所有示例

```bash
# 检查所有示例是否能编译
cargo check --examples

# 编译所有示例
cargo build --examples

# 编译 release 版本
cargo build --examples --release
```

---

## ❓ 常见问题

### 1. 如何获取 API 密钥？

**DeepSeek:**
- 访问 [DeepSeek 平台](https://platform.deepseek.com/)
- 注册账号并登录
- 在 API Keys 页面创建新密钥

**阿里云 DashScope:**
- 访问 [阿里云 DashScope](https://dashscope.console.aliyun.com/)
- 开通服务
- 在 API-KEY 管理页面创建密钥

**智谱 AI:**
- 访问 [智谱开放平台](https://open.bigmodel.cn/)
- 注册并实名认证
- 在个人中心获取 API Key

**LongCat:**
- 访问 [LongCat 平台](https://longcat.chat/platform/)
- 注册账号
- 在 API Keys 页面创建密钥
- 免费额度：500,000 tokens/天

### 2. 示例运行失败怎么办？

**检查环境变量：**
```bash
# 查看是否设置了环境变量
echo $DEEPSEEK_API_KEY

# 如果为空，需要先设置
export DEEPSEEK_API_KEY="your-api-key"
```

**检查 API 密钥是否有效：**
- 确认密钥没有过期
- 确认账户有足够的余额或额度
- 确认密钥有正确的权限

**查看详细错误信息：**
```bash
RUST_LOG=debug cargo run --example deepseek_example
```

### 3. 如何添加新的提供商？

参考 `protocol_architecture_demo.rs` 示例，了解如何：
1. 选择合适的协议（OpenAI、Anthropic 或 Aliyun）
2. 创建提供商配置
3. 使用工厂模式创建提供商实例

### 4. 可以同时使用多个提供商吗？

可以！参考 `test_all_providers.rs` 示例，它展示了如何：
- 配置多个提供商
- 在运行时切换提供商
- 管理多个 API 密钥

### 5. 示例中的配置可以用于生产环境吗？

示例代码主要用于学习和测试。在生产环境中，建议：
- 使用更完善的错误处理
- 添加日志和监控
- 实现重试机制
- 使用配置文件管理密钥
- 添加速率限制

---

## 📖 更多资源

- [项目主页](https://github.com/lipish/llm-connector)
- [API 文档](../docs/specs/API.md)
- [配置指南](../docs/CONFIGURATION_GUIDE.md)
- [架构设计](../docs/ARCHITECTURE_DESIGN.md)

---

## 🤝 贡献

如果您有新的示例想法或发现了问题，欢迎：
- 提交 Issue
- 发起 Pull Request
- 分享您的使用经验

---

## 📄 许可证

MIT License - 详见 [LICENSE](../LICENSE) 文件

