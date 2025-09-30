# llm-connector Documentation

欢迎来到 llm-connector 文档！这是一个专注于协议适配的轻量级 LLM 连接库。

## 📚 文档目录

### 核心文档

- **[简化设计](./SIMPLE_DESIGN.md)** - 项目架构和设计哲学
- **[API 规范](./specs/API.md)** - 统一接口和数据格式定义
- **[配置规范](./specs/CONFIG_SCHEMA.md)** - 配置方式和最佳实践
- **[错误处理](./specs/ERRORS.md)** - 错误分类和处理策略

### 扩展指南

- **[提供商扩展](./specs/PROVIDER_EXTENSION.md)** - 如何添加新的 LLM 提供商

## 🎯 项目定位

llm-connector 是一个**专注于协议适配**的轻量级库，遵循 Unix 哲学："Do one thing and do it well"。

### ✅ 我们做什么

- **协议转换**：将不同 LLM 提供商的 API 转换为统一的 OpenAI 兼容格式
- **提供商抽象**：通过统一接口访问多个 LLM 提供商
- **错误标准化**：统一的错误处理和重试策略
- **类型安全**：强类型定义，编译时检查

### ❌ 我们不做什么

- **负载均衡**：使用 nginx、HAProxy 等专业工具
- **熔断器**：使用基础设施层解决方案
- **健康检查**：使用外部监控系统
- **请求队列**：使用消息队列系统
- **指标收集**：使用 Prometheus 等监控工具
- **复杂路由**：保持简单的模型选择逻辑

## 🚀 快速开始

### 安装

```toml
[dependencies]
llm-connector = "0.1.0"
```

### 基本使用

```rust
use llm_connector::{Client, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量初始化
    let client = Client::from_env();
    
    // 发送请求
    let response = client.chat(ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello!".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }).await?;
    
    println!("Response: {}", response.choices[0].message.content);
    Ok(())
}
```

### 环境变量配置

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# DeepSeek
export DEEPSEEK_API_KEY="sk-..."

# 其他提供商...
```

## 🏗️ 架构概览

```
Client → Provider Registry → Providers (OpenAI/Anthropic/DeepSeek/...)
```

### 核心组件

1. **Client** - 主要接口，提供统一的 API
2. **Provider Registry** - 管理已配置的提供商
3. **Providers** - 具体的提供商实现
4. **Type System** - OpenAI 兼容的类型定义
5. **Error Handling** - 统一的错误处理

## 🔧 支持的提供商

| 提供商 | 状态 | 模型示例 |
|--------|------|----------|
| OpenAI | ✅ 计划中 | `gpt-4`, `gpt-3.5-turbo` |
| Anthropic | ✅ 计划中 | `claude-3-haiku`, `claude-3-sonnet` |
| DeepSeek | ✅ 计划中 | `deepseek-chat`, `deepseek-coder` |
| GLM (智谱) | ✅ 计划中 | `glm-4`, `chatglm3-6b` |
| Qwen (阿里) | ✅ 计划中 | `qwen-turbo`, `qwen-plus` |
| Kimi (月之暗面) | ✅ 计划中 | `moonshot-v1-8k`, `moonshot-v1-32k` |

## 📖 详细文档

### 设计文档

- **[简化设计](./SIMPLE_DESIGN.md)** - 了解项目的设计理念和架构决策

### API 文档

- **[API 规范](./specs/API.md)** - 完整的 API 接口文档
  - Client 接口
  - 请求/响应类型
  - 流式处理
  - 模型命名规范

### 配置文档

- **[配置规范](./specs/CONFIG_SCHEMA.md)** - 配置方式和最佳实践
  - 环境变量配置
  - 代码配置
  - 安全最佳实践

### 错误处理

- **[错误处理](./specs/ERRORS.md)** - 错误分类和处理策略
  - 错误类型定义
  - 重试策略
  - 调试指南

### 扩展指南

- **[提供商扩展](./specs/PROVIDER_EXTENSION.md)** - 如何添加新的提供商
  - Provider trait 实现
  - 协议转换
  - 测试指南

## 🤝 贡献指南

### 添加新提供商

1. 阅读 [提供商扩展指南](./specs/PROVIDER_EXTENSION.md)
2. 实现 Provider trait
3. 添加配置支持
4. 编写测试
5. 更新文档

### 报告问题

- 使用 GitHub Issues
- 提供详细的错误信息
- 包含复现步骤

### 提交代码

- Fork 项目
- 创建功能分支
- 编写测试
- 提交 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](../LICENSE) 文件了解详情。

## 🔗 相关链接

- [GitHub 仓库](https://github.com/lipish/llm-connector)
- [Crates.io](https://crates.io/crates/llm-connector) (计划中)
- [文档网站](https://docs.rs/llm-connector) (计划中)

## 📞 联系我们

- GitHub Issues: [问题反馈](https://github.com/lipish/llm-connector/issues)
- 邮箱: lipeng.sh@qq.com

---

**llm-connector** - 简单、专注、可靠的 LLM 协议适配库
