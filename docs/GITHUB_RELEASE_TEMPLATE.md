# llm-connector v0.5.0 - Native Multi-modal Content Support

## 🎉 重大更新

llm-connector v0.5.0 引入了**原生多模态内容支持**，允许在单个消息中同时发送文本和图片！

---

## ⚠️ Breaking Changes

### Message.content 类型变更

```rust
// v0.4.x
pub struct Message {
    pub content: String,  // 只支持文本
}

// v0.5.0
pub struct Message {
    pub content: Vec<MessageBlock>,  // 支持多模态内容
}
```

### 快速迁移

```rust
// ❌ 旧代码
let message = Message {
    role: Role::User,
    content: "Hello".to_string(),
    ..Default::default()
};

// ✅ 新代码
let message = Message::text(Role::User, "Hello");
```

📖 **完整迁移指南**: [MIGRATION_GUIDE_v0.5.0.md](https://github.com/lipish/llm-connector/blob/main/docs/MIGRATION_GUIDE_v0.5.0.md)

---

## 🎨 新功能

### 1. 多模态内容支持

```rust
use llm_connector::{LlmClient, types::{Message, MessageBlock, Role}};

let client = LlmClient::openai("sk-...")?;

// 文本 + 图片
let message = Message::new(
    Role::User,
    vec![
        MessageBlock::text("What's in this image?"),
        MessageBlock::image_url("https://example.com/image.jpg"),
    ],
);
```

### 2. MessageBlock API

```rust
// 文本块
MessageBlock::text("Hello")

// 图片 URL
MessageBlock::image_url("https://example.com/image.jpg")

// Base64 图片
MessageBlock::image_base64("image/jpeg", base64_data)
```

### 3. 便捷构造函数

```rust
// 简洁的消息创建
Message::text(Role::User, "Hello")
Message::system("You are helpful")
Message::user("Hello")
Message::assistant("Hi!")

// 辅助方法
message.content_as_text()
message.is_text_only()
message.has_images()
```

---

## 🔧 改进

### 代码清理

- 删除 27 个重复/调试示例（减少 69%）
- 删除 9 个过时的 shell 测试脚本
- 保留 12 个精选示例

### 文档更新

- ✅ 新增多模态示例
- ✅ 更新所有示例代码
- ✅ 添加编码规范文档
- ✅ 完整的迁移指南

### 测试覆盖

- ✅ 221 个测试全部通过
- ✅ 100% 测试通过率
- ✅ 0 编译错误
- ✅ 0 编译警告

---

## 📊 统计数据

| 指标 | v0.4.x | v0.5.0 | 变化 |
|------|--------|--------|------|
| Examples | 39 个 | 12 个 | -69% ↓ |
| Tests | 18 个 | 8 个 | -56% ↓ |
| 测试数量 | 213 个 | 221 个 | +8 个 ↑ |
| 编译错误 | 0 | 0 | ✅ |
| 编译警告 | 5 个 | 0 | ✅ |

---

## 🚀 快速开始

### 安装

```toml
[dependencies]
llm-connector = "0.5.0"
```

### 基础使用

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::openai("sk-...")?;
    
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("Hello!")],
        ..Default::default()
    };
    
    let response = client.chat(&request).await?;
    println!("{}", response.content);
    Ok(())
}
```

### 多模态示例

```rust
use llm_connector::types::{Message, MessageBlock, Role};

let message = Message::new(
    Role::User,
    vec![
        MessageBlock::text("What's in this image?"),
        MessageBlock::image_url("https://example.com/image.jpg"),
    ],
);
```

---

## 📝 完整变更日志

查看 [CHANGELOG.md](https://github.com/lipish/llm-connector/blob/main/CHANGELOG.md) 获取完整的变更列表。

---

## 🔗 资源链接

- **GitHub**: https://github.com/lipish/llm-connector
- **Crates.io**: https://crates.io/crates/llm-connector
- **文档**: https://docs.rs/llm-connector
- **示例**: https://github.com/lipish/llm-connector/tree/main/examples
- **迁移指南**: [MIGRATION_GUIDE_v0.5.0.md](https://github.com/lipish/llm-connector/blob/main/docs/MIGRATION_GUIDE_v0.5.0.md)

---

## 💡 支持的 Provider

- ✅ **OpenAI** - 完整支持（文本 + 图片）
- ✅ **Anthropic** - 完整支持（文本 + 图片）
- ⚠️ **其他 Providers** - 仅文本（图片转换为文本描述）

---

## 🙏 致谢

感谢所有使用 llm-connector 的开发者！

如有问题或建议，请在 GitHub 上提 issue。

---

**llm-connector v0.5.0 - 原生多模态内容支持！** 🎊

