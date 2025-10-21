# 多模态内容支持设计方案

## 📋 问题描述

当前 `Message.content` 只支持纯文本（`String`），无法处理多模态内容（图片、文档等）。

### 当前结构

```rust
pub struct Message {
    pub role: Role,
    pub content: String,  // ❌ 只支持纯文本
    // ...
}
```

### 实际 API 格式

**Anthropic API**:
```json
{
  "messages": [{
    "role": "user",
    "content": [
      {"type": "text", "text": "What's in this image?"},
      {"type": "image", "source": {"type": "base64", "media_type": "image/jpeg", "data": "..."}}
    ]
  }]
}
```

**OpenAI API**:
```json
{
  "messages": [{
    "role": "user",
    "content": [
      {"type": "text", "text": "What's in this image?"},
      {"type": "image_url", "image_url": {"url": "https://..."}}
    ]
  }]
}
```

---

## 🎯 设计目标

1. ✅ 支持多模态内容（文本、图片、文档）
2. ✅ 向后兼容现有代码
3. ✅ 统一的 API 接口
4. ✅ 类型安全
5. ✅ 易于使用

---

## 🔧 解决方案

### 方案 1: 使用枚举（推荐）⭐⭐⭐⭐⭐

**原理**: 将 `content` 改为枚举类型，支持多种内容格式

**实现**:

```rust
// src/types/content.rs

/// 消息内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// 纯文本内容（向后兼容）
    Text(String),
    
    /// 多模态内容数组
    Parts(Vec<ContentPart>),
}

/// 内容部分
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// 文本内容
    Text {
        text: String,
    },
    
    /// 图片内容（Anthropic 格式）
    Image {
        source: ImageSource,
    },
    
    /// 图片 URL（OpenAI 格式）
    ImageUrl {
        image_url: ImageUrl,
    },
    
    /// 文档内容（Anthropic）
    Document {
        source: DocumentSource,
    },
}

/// 图片来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64 编码
    Base64 {
        media_type: String,
        data: String,
    },
    
    /// URL
    Url {
        url: String,
    },
}

/// 图片 URL（OpenAI 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,  // "auto", "low", "high"
}

/// 文档来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentSource {
    /// Base64 编码
    Base64 {
        media_type: String,
        data: String,
    },
}

// 更新 Message 结构
pub struct Message {
    pub role: Role,
    pub content: MessageContent,  // ✅ 支持多模态
    // ...
}
```

**优点**:
- ✅ 类型安全
- ✅ 支持所有主流格式
- ✅ 易于扩展
- ✅ 编译时检查

**缺点**:
- ⚠️ 需要迁移现有代码

---

### 方案 2: 使用 serde_json::Value（灵活）⭐⭐⭐

**原理**: 使用动态 JSON 值

**实现**:

```rust
use serde_json::Value;

pub struct Message {
    pub role: Role,
    pub content: Value,  // 可以是 String 或 Array
    // ...
}
```

**优点**:
- ✅ 极其灵活
- ✅ 易于实现

**缺点**:
- ❌ 无类型安全
- ❌ 运行时错误
- ❌ 难以使用

---

### 方案 3: 添加新字段（向后兼容）⭐⭐⭐⭐

**原理**: 保留 `content: String`，添加 `content_parts: Option<Vec<ContentPart>>`

**实现**:

```rust
pub struct Message {
    pub role: Role,
    
    /// 纯文本内容（向后兼容）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    
    /// 多模态内容（新增）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_parts: Option<Vec<ContentPart>>,
    
    // ...
}
```

**优点**:
- ✅ 完全向后兼容
- ✅ 渐进式迁移

**缺点**:
- ⚠️ 两个字段可能冲突
- ⚠️ 需要处理优先级

---

## 📊 方案对比

| 方案 | 类型安全 | 向后兼容 | 易用性 | 推荐度 |
|------|----------|----------|--------|--------|
| **方案 1: 枚举** | ✅ 高 | ⚠️ 需迁移 | ✅ 好 | ⭐⭐⭐⭐⭐ |
| 方案 2: Value | ❌ 低 | ✅ 完全 | ⚠️ 一般 | ⭐⭐⭐ |
| 方案 3: 新字段 | ✅ 高 | ✅ 完全 | ⚠️ 一般 | ⭐⭐⭐⭐ |

---

## 🎯 推荐实现：方案 1（枚举）+ 向后兼容

### 核心思路

1. 使用 `MessageContent` 枚举支持多模态
2. 提供便捷方法保持向后兼容
3. 自动序列化/反序列化

### 详细实现

#### 1. 定义内容类型

```rust
// src/types/content.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// 纯文本（向后兼容）
    Text(String),
    
    /// 多模态内容
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    /// 创建纯文本内容
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }
    
    /// 创建多模态内容
    pub fn parts(parts: Vec<ContentPart>) -> Self {
        Self::Parts(parts)
    }
    
    /// 获取纯文本（如果是纯文本）
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Parts(parts) if parts.len() == 1 => {
                match &parts[0] {
                    ContentPart::Text { text } => Some(text),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    /// 转换为纯文本（提取所有文本部分）
    pub fn to_text(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Parts(parts) => {
                parts.iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}
```

#### 2. 定义内容部分

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    
    Image {
        source: ImageSource,
    },
    
    ImageUrl {
        image_url: ImageUrl,
    },
}

impl ContentPart {
    /// 创建文本部分
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
    
    /// 创建 Base64 图片
    pub fn image_base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Image {
            source: ImageSource::Base64 {
                media_type: media_type.into(),
                data: data.into(),
            },
        }
    }
    
    /// 创建图片 URL（OpenAI 格式）
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: None,
            },
        }
    }
}
```

#### 3. 更新 Message

```rust
// src/types/request.rs

pub struct Message {
    pub role: Role,
    pub content: MessageContent,  // ✅ 支持多模态
    // ... 其他字段
}

impl Message {
    /// 创建纯文本消息（向后兼容）
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: MessageContent::text(content),
            ..Default::default()
        }
    }
    
    /// 创建多模态消息
    pub fn with_parts(role: Role, parts: Vec<ContentPart>) -> Self {
        Self {
            role,
            content: MessageContent::parts(parts),
            ..Default::default()
        }
    }
}
```

---

## 🧪 使用示例

### 示例 1: 纯文本（向后兼容）

```rust
// 方式 1: 直接使用字符串
let message = Message {
    role: Role::User,
    content: "Hello".into(),
    ..Default::default()
};

// 方式 2: 使用构造函数
let message = Message::new(Role::User, "Hello");
```

### 示例 2: 图片 + 文本（Anthropic 格式）

```rust
use llm_connector::types::{Message, Role, ContentPart};

let message = Message::with_parts(
    Role::User,
    vec![
        ContentPart::text("What's in this image?"),
        ContentPart::image_base64(
            "image/jpeg",
            "iVBORw0KGgoAAAANSUhEUgA..."
        ),
    ],
);
```

### 示例 3: 图片 URL（OpenAI 格式）

```rust
let message = Message::with_parts(
    Role::User,
    vec![
        ContentPart::text("Describe this image"),
        ContentPart::image_url("https://example.com/image.jpg"),
    ],
);
```

---

## 🔄 迁移指南

### 现有代码

```rust
// 旧代码
let message = Message {
    role: Role::User,
    content: "Hello".to_string(),
    ..Default::default()
};
```

### 迁移后

```rust
// 新代码（完全兼容）
let message = Message {
    role: Role::User,
    content: "Hello".into(),  // 自动转换为 MessageContent::Text
    ..Default::default()
};

// 或使用构造函数
let message = Message::new(Role::User, "Hello");
```

**无需修改现有代码！**

---

## 📝 实现清单

- [ ] 创建 `src/types/content.rs`
- [ ] 定义 `MessageContent` 枚举
- [ ] 定义 `ContentPart` 枚举
- [ ] 定义 `ImageSource`、`ImageUrl` 等
- [ ] 更新 `Message` 结构
- [ ] 添加便捷构造函数
- [ ] 更新 Anthropic protocol
- [ ] 更新 OpenAI protocol
- [ ] 添加单元测试
- [ ] 添加示例代码
- [ ] 更新文档

---

## 🎯 总结

**推荐方案**: 方案 1（枚举）+ 向后兼容

**核心优势**:
1. ✅ **类型安全** - 编译时检查
2. ✅ **向后兼容** - 通过 `From<String>` 和 `#[serde(untagged)]`
3. ✅ **易于使用** - 便捷构造函数
4. ✅ **统一接口** - 支持所有 providers
5. ✅ **可扩展** - 易于添加新类型

**实现步骤**:
1. 创建新的内容类型定义
2. 更新 Message 结构
3. 提供向后兼容的 API
4. 更新 protocols 以支持多模态
5. 添加测试和文档

**预期效果**:
- 现有代码无需修改
- 新代码可以使用多模态功能
- 统一的 API 接口
- 完整的类型安全

