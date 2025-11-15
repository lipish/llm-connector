# LLM Connector Examples

这个目录包含了 `llm-connector` 库的精选使用示例。

## 📚 示例列表

### 基础示例 (7 个)

| 示例文件 | 描述 | 运行命令 |
|---------|------|----------|
| `openai_basic.rs` | OpenAI 基础聊天示例 | `cargo run --example openai_basic` |
| `aliyun_basic.rs` | 阿里云通义千问基础示例 | `cargo run --example aliyun_basic` |
| `zhipu_basic.rs` | 智谱 GLM 基础示例 | `cargo run --example zhipu_basic` |
| `tencent_basic.rs` | 腾讯混元基础示例 | `cargo run --example tencent_basic` |
| `ollama_basic.rs` | Ollama 本地模型基础示例 | `cargo run --example ollama_basic` |
| `anthropic_streaming.rs` | Anthropic 流式响应示例 | `cargo run --example anthropic_streaming --features streaming` |
| `volcengine_streaming.rs` | Volcengine 流式响应示例（支持推理模型） | `cargo run --example volcengine_streaming --features streaming -- <api-key> <endpoint>` |

### 特殊功能 (4 个)

| 示例文件 | 描述 | 运行命令 |
|---------|------|----------|
| `multimodal_basic.rs` | 多模态内容示例（文本+图片） | `cargo run --example multimodal_basic` |
| `ollama_model_management.rs` | Ollama 模型管理（CRUD） | `cargo run --example ollama_model_management` |
| `ollama_streaming.rs` | Ollama 流式响应示例 | `cargo run --example ollama_streaming --features streaming` |
| `aliyun_thinking.rs` | Aliyun thinking 功能示例 | `cargo run --example aliyun_thinking` |

### 工具调用 (2 个)

| 示例文件 | 描述 | 运行命令 |
|---------|------|----------|
| `zhipu_tools.rs` | 智谱 GLM 工具调用基础示例 | `cargo run --example zhipu_tools` |
| `zhipu_multiround_tools.rs` | 智谱 GLM 多轮工具调用示例 | `cargo run --example zhipu_multiround_tools` |

## 🔧 环境变量设置

### OpenAI
```bash
export OPENAI_API_KEY="your-openai-api-key"
```

### 阿里云DashScope
```bash
export DASHSCOPE_API_KEY="your-dashscope-api-key"
```

### 智谱GLM
```bash
export ZHIPU_API_KEY="your-zhipu-api-key"
```

### 腾讯混元
```bash
export TENCENT_API_KEY="your-tencent-api-key"
```

### Anthropic
```bash
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

### Ollama
```bash
# Ollama默认运行在 localhost:11434，无需API密钥
# 可选：指定模型
export OLLAMA_MODEL="llama2"
```

### Volcengine (火山引擎)
```bash
export VOLCENGINE_API_KEY="your-volcengine-api-key"
export VOLCENGINE_ENDPOINT="ep-20250118155555-xxxxx"  # 推理接入点 ID
```

## 📋 功能特性

### 支持的 Provider

- **OpenAI** - GPT 系列模型
- **阿里云 DashScope** - 通义千问系列
- **智谱 GLM** - GLM 系列模型
- **腾讯混元** - 混元系列模型
- **Anthropic** - Claude 系列模型
- **Ollama** - 本地开源模型
- **Volcengine (火山引擎)** - 豆包系列模型（包括推理模型 Doubao-Seed-Code）

### 核心功能

- ✅ 统一的聊天接口
- ✅ 流式响应支持
- ✅ 多模态内容（文本 + 图片）
- ✅ 工具调用（Function Calling）
- ✅ 模型列表获取
- ✅ Token 使用统计
- ✅ 错误处理和重试

## 🎯 快速开始

1. **从基础示例开始**：
   ```bash
   cargo run --example ollama_basic
   ```

2. **尝试多模态内容**：
   ```bash
   cargo run --example multimodal_basic
   ```

3. **尝试工具调用**：
   ```bash
   cargo run --example zhipu_tools
   ```

## 💡 提示

- 大部分示例需要相应的 API 密钥
- Ollama 示例需要本地运行 Ollama 服务
- 流式示例需要启用 `streaming` 功能
- 多模态示例需要支持视觉的模型（如 gpt-4o, claude-3-5-sonnet）

## 🔗 相关链接

- [项目主页](https://github.com/lipish/llm-connector)
- [API文档](https://docs.rs/llm-connector)
- [Crates.io](https://crates.io/crates/llm-connector)
