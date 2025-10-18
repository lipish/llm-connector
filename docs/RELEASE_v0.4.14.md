# Release v0.4.14 - 发布总结

## 📦 发布信息

- **版本**: v0.4.14
- **发布日期**: 2025-10-18
- **GitHub Tag**: https://github.com/lipish/llm-connector/releases/tag/v0.4.14
- **Crates.io**: https://crates.io/crates/llm-connector/0.4.14

## 🎯 主要更新

### 1. ✅ 完整支持 OpenAI 协议工具调用

**问题描述**:
- ❌ `OpenAIRequest` 缺少 `tools` 和 `tool_choice` 字段
- ❌ `OpenAIMessage` 缺少 `tool_calls`, `tool_call_id`, `name` 字段
- ❌ 导致 DeepSeek, Moonshot 等服务完全无法使用工具调用

**修复内容**:
- ✅ 添加所有必需的工具调用字段
- ✅ 完整实现请求构建和响应解析
- ✅ 所有 OpenAI 兼容服务现在都支持工具调用

**影响的服务**:
- DeepSeek
- Moonshot
- Together AI
- 所有其他 OpenAI 兼容服务

### 2. ✅ 移除智谱 GLM 流式响应限制

**问题描述**:
- ❌ 之前代码检测到 `Role::Tool` 消息时强制切换为非流式
- ❌ GLM-4.5 从 91 块降为 1 块

**修复内容**:
- ✅ 移除强制切换逻辑
- ✅ 智谱 GLM 现在可以在包含工具结果时正常使用流式响应

**测试结果**:
- glm-4-flash: 27 块, 128 字符 ✅
- glm-4: 32 块, 146 字符 ✅
- glm-4.5: 96 块, 267 字符 ✅
- glm-4.6: 99 块, 246 字符 ✅

## 📊 测试覆盖

### 单元测试
```bash
cargo test --lib --features streaming
# 结果: 27/27 测试通过
```

### 集成测试

#### 1. OpenAI 协议验证
```bash
cargo run --example verify_tool_fix
```
- ✅ ChatRequest 支持工具调用字段
- ✅ OpenAI 协议支持工具调用
- ✅ 智谱 GLM 流式修复已移除

#### 2. 智谱 GLM 流式响应测试
```bash
ZHIPU_API_KEY="..." cargo run --example test_zhipu_tool_streaming_issue --features streaming
```
- ✅ 所有模型在包含 Role::Tool 时正常流式响应
- ✅ 没有空内容或单块响应问题

#### 3. 详细测试
```bash
ZHIPU_API_KEY="..." cargo run --example test_zhipu_tool_messages_detailed --features streaming
```
- ✅ 详细输出每个流式块
- ✅ 验证内容完整性

## 📝 新增文件

### 测试示例
- `examples/verify_tool_fix.rs` - 验证工具调用修复
- `examples/test_zhipu_tool_streaming_issue.rs` - 测试智谱流式响应
- `examples/test_zhipu_tool_messages_detailed.rs` - 详细测试
- `examples/test_deepseek_tools.rs` - DeepSeek 工具调用测试
- `examples/test_openai_tool_streaming.rs` - OpenAI 工具调用流式测试
- `examples/test_glm_models_tool_streaming.rs` - GLM 模型测试
- `examples/test_all_providers_tool_streaming.rs` - 所有提供商测试

### 文档
- `docs/TESTING_INSTRUCTIONS.md` - 测试说明
- `docs/TEST_REPORT.md` - 测试报告
- `docs/TEST_ZHIPU_STREAMING.md` - 智谱流式测试文档

## 🔧 修改的文件

### 核心代码
- `src/protocols/openai.rs` - 添加工具调用支持
- `src/core/traits.rs` - 移除智谱 GLM 强制切换逻辑
- `src/types/streaming.rs` - 改进流式响应处理

### 配置文件
- `Cargo.toml` - 版本更新到 0.4.14
- `CHANGELOG.md` - 添加详细的更新日志
- `.gitignore` - 更新忽略规则

## 🚀 发布流程

### 1. 提交代码
```bash
git add -A
git commit -m "feat: 完整支持 OpenAI 协议工具调用 + 移除智谱 GLM 流式限制"
git push origin main
```

### 2. 发布到 crates.io
```bash
bash scripts/release.sh publish
```

### 3. 创建 Git Tag
```bash
git tag -a "v0.4.14" -m "Release v0.4.14"
git push origin v0.4.14
```

### 4. 验证发布
```bash
bash scripts/release.sh check
# Local version:  0.4.14
# Remote version: 0.4.14
```

## ✅ 验证清单

- [x] 所有单元测试通过
- [x] 所有集成测试通过
- [x] 编译无错误无警告
- [x] OpenAI 协议工具调用正常工作
- [x] 智谱 GLM 流式响应正常工作
- [x] DeepSeek 工具调用正常工作
- [x] 代码已提交到 GitHub
- [x] 已发布到 crates.io
- [x] Git tag 已创建并推送
- [x] 版本号一致（本地 = 远程）

## 📖 使用示例

### OpenAI 协议工具调用
```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}};

let client = LlmClient::deepseek("your-api-key")?;

let tools = vec![Tool {
    tool_type: "function".to_string(),
    function: Function {
        name: "get_weather".to_string(),
        description: Some("获取天气信息".to_string()),
        parameters: json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        }),
    },
}];

let request = ChatRequest {
    model: "deepseek-chat".to_string(),
    messages: vec![Message {
        role: Role::User,
        content: "上海的天气怎么样？".to_string(),
        ..Default::default()
    }],
    tools: Some(tools),
    ..Default::default()
};

let response = client.chat(&request).await?;
```

### 智谱 GLM 流式工具调用
```rust
use llm_connector::LlmClient;
use futures_util::StreamExt;

let client = LlmClient::zhipu("your-api-key")?;

// 第一轮：触发工具调用
let mut stream = client.chat_stream(&request1).await?;
while let Some(chunk) = stream.next().await {
    // 处理流式响应
}

// 第二轮：包含 Role::Tool 消息
let mut stream = client.chat_stream(&request2).await?;
while let Some(chunk) = stream.next().await {
    // ✅ 现在可以正常流式响应！
}
```

## 🎉 总结

v0.4.14 是一个重要的修复版本，解决了两个关键问题：

1. **OpenAI 协议工具调用支持** - 使所有 OpenAI 兼容服务都能使用工具调用
2. **智谱 GLM 流式限制移除** - 提升流式响应性能和用户体验

所有测试通过，完全向后兼容，可以安全升级。

---

**发布人**: AI Assistant  
**发布时间**: 2025-10-18  
**发布状态**: ✅ 成功

