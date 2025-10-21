# llm-connector v0.5.0 迁移指南

## 概述

llm-connector v0.5.0 引入了**原生多模态内容支持**，这是一个 Breaking Change。本指南将帮助你快速迁移代码。

---

## 核心变更

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

---

## 迁移步骤

### 1. 创建消息

#### ❌ 旧代码 (v0.4.x)

```rust
let message = Message {
    role: Role::User,
    content: "Hello, how are you?".to_string(),
    ..Default::default()
};
```

#### ✅ 新代码 (v0.5.0) - 推荐方式

```rust
// 方式 1: 使用便捷构造函数（推荐）
let message = Message::text(Role::User, "Hello, how are you?");

// 方式 2: 使用角色特定方法
let message = Message::user("Hello, how are you?");

// 方式 3: 使用 new() 方法
let message = Message::new(
    Role::User,
    vec![MessageBlock::text("Hello, how are you?")],
);
```

### 2. 提取文本内容

#### ❌ 旧代码 (v0.4.x)

```rust
let text = message.content;
println!("{}", response.choices[0].message.content);
```

#### ✅ 新代码 (v0.5.0)

```rust
let text = message.content_as_text();
println!("{}", response.choices[0].message.content_as_text());
```

### 3. 创建请求

#### ❌ 旧代码 (v0.4.x)

```rust
let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![
        Message {
            role: Role::System,
            content: "You are a helpful assistant.".to_string(),
            ..Default::default()
        },
        Message {
            role: Role::User,
            content: "Hello!".to_string(),
            ..Default::default()
        },
    ],
    ..Default::default()
};
```

#### ✅ 新代码 (v0.5.0)

```rust
let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![
        Message::system("You are a helpful assistant."),
        Message::user("Hello!"),
    ],
    ..Default::default()
};
```

### 4. 工具调用消息

#### ❌ 旧代码 (v0.4.x)

```rust
// Assistant 消息（带工具调用）
Message {
    role: Role::Assistant,
    content: String::new(),
    tool_calls: Some(vec![tool_call]),
    ..Default::default()
}

// Tool 消息（工具结果）
Message {
    role: Role::Tool,
    content: r#"{"result": "success"}"#.to_string(),
    tool_call_id: Some("call_123".to_string()),
    name: Some("get_weather".to_string()),
    ..Default::default()
}
```

#### ✅ 新代码 (v0.5.0)

```rust
// Assistant 消息（带工具调用）
Message {
    role: Role::Assistant,
    content: vec![],  // 空内容
    tool_calls: Some(vec![tool_call]),
    ..Default::default()
}

// Tool 消息（工具结果）
Message {
    role: Role::Tool,
    content: vec![MessageBlock::text(r#"{"result": "success"}"#)],
    tool_call_id: Some("call_123".to_string()),
    name: Some("get_weather".to_string()),
    ..Default::default()
}
```

---

## 新功能：多模态内容

### 发送图片

```rust
use llm_connector::types::{Message, MessageBlock, Role};

// 文本 + 图片 URL
let message = Message::new(
    Role::User,
    vec![
        MessageBlock::text("What's in this image?"),
        MessageBlock::image_url("https://example.com/image.jpg"),
    ],
);

// 文本 + Base64 图片
let message = Message::new(
    Role::User,
    vec![
        MessageBlock::text("Analyze this image"),
        MessageBlock::image_base64("image/jpeg", base64_data),
    ],
);
```

### MessageBlock API

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

---

## 常见模式

### 1. 简单聊天

```rust
// v0.4.x
let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![
        Message {
            role: Role::User,
            content: "Hello".to_string(),
            ..Default::default()
        }
    ],
    ..Default::default()
};

// v0.5.0
let request = ChatRequest {
    model: "gpt-4".to_string(),
    messages: vec![Message::user("Hello")],
    ..Default::default()
};
```

### 2. 系统提示 + 用户消息

```rust
// v0.4.x
let messages = vec![
    Message {
        role: Role::System,
        content: "You are helpful.".to_string(),
        ..Default::default()
    },
    Message {
        role: Role::User,
        content: "Hello".to_string(),
        ..Default::default()
    },
];

// v0.5.0
let messages = vec![
    Message::system("You are helpful."),
    Message::user("Hello"),
];
```

### 3. 多轮对话

```rust
// v0.4.x
let messages = vec![
    Message {
        role: Role::User,
        content: "What's the weather?".to_string(),
        ..Default::default()
    },
    Message {
        role: Role::Assistant,
        content: "It's sunny.".to_string(),
        ..Default::default()
    },
    Message {
        role: Role::User,
        content: "Thanks!".to_string(),
        ..Default::default()
    },
];

// v0.5.0
let messages = vec![
    Message::user("What's the weather?"),
    Message::assistant("It's sunny."),
    Message::user("Thanks!"),
];
```

---

## 辅助方法

### 检查消息类型

```rust
// 检查是否只包含文本
if message.is_text_only() {
    println!("纯文本消息");
}

// 检查是否包含图片
if message.has_images() {
    println!("包含图片");
}

// 提取文本内容
let text = message.content_as_text();
```

---

## 自动化迁移

### 使用 sed 批量替换

```bash
# 替换 Message 构造
find src -name "*.rs" -exec sed -i '' 's/Message {$/Message::text(/g' {} \;

# 替换 content 访问
find src -name "*.rs" -exec sed -i '' 's/\.content/.content_as_text()/g' {} \;
```

### 使用 IDE 重构

大多数 IDE 支持批量重构：

1. 查找 `Message {`
2. 替换为 `Message::text(`
3. 查找 `.content`
4. 替换为 `.content_as_text()`

---

## 测试迁移

### 更新测试代码

```rust
// v0.4.x
#[test]
fn test_message() {
    let message = Message {
        role: Role::User,
        content: "Hello".to_string(),
        ..Default::default()
    };
    assert_eq!(message.content, "Hello");
}

// v0.5.0
#[test]
fn test_message() {
    let message = Message::text(Role::User, "Hello");
    assert_eq!(message.content_as_text(), "Hello");
}
```

---

## 常见问题

### Q: 为什么要做这个 Breaking Change？

A: 为了支持多模态内容（文本 + 图片），这是现代 LLM 的核心功能。

### Q: 迁移成本高吗？

A: 不高。使用便捷构造函数，大部分代码只需要简单替换。

### Q: 所有 Provider 都支持多模态吗？

A: 目前 OpenAI 和 Anthropic 完整支持。其他 Provider 会将图片转换为文本描述。

### Q: 如何处理旧代码？

A: 使用 `Message::text()` 和 `content_as_text()` 即可快速迁移。

---

## 检查清单

迁移完成后，检查以下项目：

- [ ] 所有 `Message { role, content: "...".to_string(), ... }` 已替换
- [ ] 所有 `message.content` 已替换为 `message.content_as_text()`
- [ ] 所有测试通过
- [ ] 代码编译无错误
- [ ] 代码编译无警告

---

## 获取帮助

如果遇到问题：

1. 查看 [示例代码](https://github.com/lipish/llm-connector/tree/main/examples)
2. 阅读 [API 文档](https://docs.rs/llm-connector)
3. 在 GitHub 上提 [Issue](https://github.com/lipish/llm-connector/issues)

---

**祝迁移顺利！** 🚀

