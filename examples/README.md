# LLM Connector Examples

这个目录包含了 `llm-connector` 库的各种使用示例。

## 🚀 基础示例

### Provider基础用法

| 示例文件 | 描述 | 运行命令 |
|---------|------|----------|
| `openai_basic.rs` | OpenAI基础聊天示例 | `cargo run --example openai_basic` |
| `aliyun_basic.rs` | 阿里云通义千问基础示例 | `cargo run --example aliyun_basic` |
| `zhipu_basic.rs` | 智谱GLM基础示例 | `cargo run --example zhipu_basic` |
| `tencent_basic.rs` | 腾讯混元基础示例 | `cargo run --example tencent_basic --features tencent` |
| `ollama_basic.rs` | Ollama本地模型基础示例 | `cargo run --example ollama_basic` |

### 流式响应示例

| 示例文件 | 描述 | 运行命令 |
|---------|------|----------|
| `streaming_basic.rs` | 通用流式响应示例 | `cargo run --example streaming_basic --features streaming` |
| `anthropic_streaming.rs` | Anthropic流式响应示例 | `cargo run --example anthropic_streaming --features streaming` |
| `zhipu_streaming.rs` | 智谱GLM流式响应示例 | `cargo run --example zhipu_streaming --features streaming` |
| `ollama_streaming.rs` | Ollama流式响应示例 | `cargo run --example ollama_streaming --features streaming` |

### 高级功能示例

| 示例文件 | 描述 | 运行命令 |
|---------|------|----------|
| `ollama_model_management.rs` | Ollama模型管理示例 | `cargo run --example ollama_model_management` |
| `test_keys_yaml.rs` | 批量测试API密钥 | `cargo run --example test_keys_yaml` |

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
export TENCENT_SECRET_ID="your-secret-id"
export TENCENT_SECRET_KEY="your-secret-key"
export TENCENT_REGION="ap-beijing"  # 可选
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

## 📋 功能特性

### 支持的Provider

- **OpenAI** - GPT系列模型
- **阿里云DashScope** - 通义千问系列
- **智谱GLM** - GLM系列模型  
- **腾讯混元** - 混元系列模型
- **Anthropic** - Claude系列模型
- **Ollama** - 本地开源模型

### 核心功能

- ✅ 统一的聊天接口
- ✅ 流式响应支持
- ✅ 模型列表获取
- ✅ Token使用统计
- ✅ 错误处理和重试
- ✅ 配置文件支持

## 🎯 快速开始

1. **选择一个基础示例**开始：
   ```bash
   cargo run --example ollama_basic
   ```

2. **尝试流式响应**：
   ```bash
   cargo run --example streaming_basic --features streaming
   ```

3. **测试多个API密钥**：
   ```bash
   cargo run --example test_keys_yaml
   ```

## 💡 提示

- 大部分示例需要相应的API密钥
- Ollama示例需要本地运行Ollama服务
- 流式示例需要启用 `streaming` 功能
- 腾讯混元示例需要启用 `tencent` 功能

## 🔗 相关链接

- [项目主页](https://github.com/lipish/llm-connector)
- [API文档](https://docs.rs/llm-connector)
- [Crates.io](https://crates.io/crates/llm-connector)
