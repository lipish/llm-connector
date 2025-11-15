# llm-connector v0.5.0 Release Notes

## 🎉 重大更新：原生多模态内容支持

llm-connector v0.5.0 是一个重大版本更新，引入了**原生多模态内容支持**，允许在单个消息中同时发送文本和图片。

---

## ⚠️ Breaking Changes

### Message.content 类型变更

**之前 (v0.4.x)**:
```rust
pub struct Message {
    pub role: Role,
    pub content: String,  // 只支持文本
    // ...
}
```

**现在 (v0.5.0)**:
```rust
pub struct Message {
    pub role: Role,
    pub content: Vec<MessageBlock>,  // 支持多模态内容
    // ...
}
```

### 迁移指南

#### 简单迁移（推荐）

使用新的便捷构造函数：

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

#### 提取文本内容

```rust
// ❌ 旧代码
let text = message.content;

// ✅ 新代码
let text = message.content_as_text();
```

---

## 🎨 新功能

### 1. 多模态内容支持

现在可以在单个消息中发送文本和图片：

```rust
use llm_connector::{LlmClient, types::{ChatRequest, Message, MessageBlock, Role}};

let client = LlmClient::openai("sk-...")?;

// 文本 + 图片 URL
let request = ChatRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        Message::new(
            Role::User,
            vec![
                MessageBlock::text("What's in this image?"),
                MessageBlock::image_url("https://example.com/image.jpg"),
            ],
        ),
    ],
    ..Default::default()
};

// 文本 + Base64 图片
let request = ChatRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        Message::new(
            Role::User,
            vec![
                MessageBlock::text("Analyze this image"),
                MessageBlock::image_base64("image/jpeg", base64_data),
            ],
        ),
    ],
    ..Default::default()
};
```

### 2. MessageBlock API

新增 `MessageBlock` 类型，支持多种内容类型：

```rust
// 文本块
MessageBlock::text("Hello")

// 图片 URL (OpenAI 格式)
MessageBlock::image_url("https://example.com/image.jpg")

// 图片 URL (带 detail 参数)
MessageBlock::image_url_with_detail("https://...", "high")

// Base64 图片
MessageBlock::image_base64("image/jpeg", base64_data)

// 图片 URL (Anthropic 格式)
MessageBlock::image_url_anthropic("https://...")
```

### 3. 便捷构造函数

新增多个便捷方法：

```rust
// Message 构造
Message::text(Role::User, "Hello")
Message::system("You are helpful")
Message::user("Hello")
Message::assistant("Hi!")
Message::new(Role::User, vec![...])

// 辅助方法
message.content_as_text()
message.is_text_only()
message.has_images()
```

### 4. Provider 支持

- ✅ **OpenAI** - 完整支持（文本 + 图片）
- ✅ **Anthropic** - 完整支持（文本 + 图片）
- ⚠️ **其他 Providers** - 仅文本（图片转换为文本描述）

---

## 🔧 改进

### 1. 代码清理

- 删除 27 个重复/调试示例文件（减少 69%）
- 删除 9 个过时的 shell 测试脚本
- 保留 12 个精选示例，每个都有明确目的

### 2. 文档更新

- ✅ 更新 README 添加多模态示例
- ✅ 新增 `examples/multimodal_basic.rs` 示例
- ✅ 新增 `docs/RUST_CODING_RULES.md` 编码规范
- ✅ 更新所有示例使用新 API

### 3. 测试覆盖

- ✅ 221 个测试全部通过
- ✅ 新增 8 个 MessageBlock 测试
- ✅ 100% 测试通过率

---

## 📊 统计数据

| 指标 | v0.4.x | v0.5.0 | 变化 |
|------|--------|--------|------|
| **Examples** | 39 个 | 12 个 | -69% ↓ |
| **Tests** | 18 个 | 8 个 | -56% ↓ |
| **测试数量** | 213 个 | 221 个 | +8 个 ↑ |
| **编译错误** | 0 | 0 | ✅ |
| **编译警告** | 5 个 | 0 | ✅ |

---

## 🚀 升级步骤

### 1. 更新依赖

```toml
[dependencies]
llm-connector = "0.5.0"
```

### 2. 更新代码

#### 简单文本消息

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

#### 提取文本内容

```rust
// ❌ 旧代码
println!("{}", response.choices[0].message.content);

// ✅ 新代码
println!("{}", response.choices[0].message.content_as_text());
```

### 3. 运行测试

```bash
cargo test
```

---

## 📝 完整变更日志

查看 [CHANGELOG.md](../CHANGELOG.md) 获取完整的变更列表。

---

## 🔗 资源链接

- **GitHub**: https://github.com/lipish/llm-connector
- **Crates.io**: https://crates.io/crates/llm-connector
- **文档**: https://docs.rs/llm-connector
- **示例**: https://github.com/lipish/llm-connector/tree/main/examples

---

## 💡 使用建议

### 对于新用户

直接使用 v0.5.0，享受多模态内容支持和更清晰的 API。

### 对于现有用户

1. 阅读迁移指南
2. 使用便捷构造函数替换旧代码
3. 使用 `content_as_text()` 提取文本
4. 运行测试确保兼容性

---

## 🙏 致谢

感谢所有使用 llm-connector 的开发者！

如有问题或建议，请在 GitHub 上提 issue。

---

**llm-connector v0.5.0 - 原生多模态内容支持！** 🎊

