# 多模态内容原生支持设计（简洁版）

## 📋 核心思路

**放弃向后兼容，直接使用最简洁的设计**

在项目前期阶段，应该：
- ✅ 采用最 native 的结构
- ✅ 减少不必要的抽象层
- ✅ 直接对齐主流 API 格式
- ❌ 不为向后兼容增加复杂度

---

## 🎯 推荐方案：直接使用 Vec<ContentPart>

### 核心设计

```rust
// src/types/content.rs

/// 消息内容部分
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// 文本内容
    Text {
        text: String,
    },
    
    /// 图片（Anthropic 格式）
    Image {
        source: ImageSource,
    },
    
    /// 图片 URL（OpenAI 格式）
    ImageUrl {
        image_url: ImageUrl,
    },
}

/// 图片来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 {
        media_type: String,
        data: String,
    },
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

// 更新 Message
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentPart>,  // ✅ 直接使用数组
    // ...
}
```

### 为什么这样设计？

1. **简洁** ✅
   - 无需 `MessageContent` 枚举
   - 无需 `#[serde(untagged)]` 复杂序列化
   - 结构清晰明了

2. **Native** ✅
   - 直接对齐 Anthropic/OpenAI API
   - 无额外抽象层
   - 序列化/反序列化直接

3. **灵活** ✅
   - 纯文本：`vec![ContentPart::text("Hello")]`
   - 多模态：`vec![ContentPart::text("..."), ContentPart::image(...)]`
   - 统一处理

---

## 📊 对比：复杂方案 vs 简洁方案

| 特性 | 复杂方案（枚举） | 简洁方案（数组） |
|------|------------------|------------------|
| **类型定义** | `MessageContent` + `ContentPart` | 只需 `ContentPart` |
| **序列化** | `#[serde(untagged)]` | 标准序列化 |
| **纯文本** | `MessageContent::Text(s)` | `vec![ContentPart::text(s)]` |
| **多模态** | `MessageContent::Parts(vec![...])` | `vec![...]` |
| **代码行数** | ~150 行 | ~80 行 |
| **复杂度** | ⚠️ 中等 | ✅ 低 |
| **维护成本** | ⚠️ 高 | ✅ 低 |

---

## 🔧 完整实现

### 1. 定义内容类型

```rust
// src/types/content.rs

use serde::{Deserialize, Serialize};

/// 消息内容部分
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// 文本内容
    Text {
        text: String,
    },
    
    /// 图片（Anthropic 格式）
    Image {
        source: ImageSource,
    },
    
    /// 图片 URL（OpenAI 格式）
    ImageUrl {
        image_url: ImageUrl,
    },
}

impl ContentPart {
    /// 创建文本部分
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
    
    /// 创建 Base64 图片（Anthropic 格式）
    pub fn image_base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Image {
            source: ImageSource::Base64 {
                media_type: media_type.into(),
                data: data.into(),
            },
        }
    }
    
    /// 创建图片 URL（Anthropic 格式）
    pub fn image_url_anthropic(url: impl Into<String>) -> Self {
        Self::Image {
            source: ImageSource::Url {
                url: url.into(),
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
    
    /// 创建图片 URL（OpenAI 格式，带 detail）
    pub fn image_url_with_detail(url: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: Some(detail.into()),
            },
        }
    }
}

/// 图片来源（Anthropic 格式）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 {
        media_type: String,
        data: String,
    },
    Url {
        url: String,
    },
}

/// 图片 URL（OpenAI 格式）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
```

### 2. 更新 Message

```rust
// src/types/request.rs

use super::content::ContentPart;

pub struct Message {
    pub role: Role,
    pub content: Vec<ContentPart>,  // ✅ 直接使用数组
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    // ... 其他字段
}

impl Message {
    /// 创建纯文本消息
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![ContentPart::text(text)],
            ..Default::default()
        }
    }
    
    /// 创建多模态消息
    pub fn new(role: Role, content: Vec<ContentPart>) -> Self {
        Self {
            role,
            content,
            ..Default::default()
        }
    }
}
```

---

## 🧪 使用示例

### 示例 1: 纯文本

```rust
// 方式 1: 使用构造函数
let message = Message::text(Role::User, "Hello");

// 方式 2: 直接构造
let message = Message {
    role: Role::User,
    content: vec![ContentPart::text("Hello")],
    ..Default::default()
};
```

### 示例 2: 图片 + 文本（Anthropic）

```rust
let message = Message::new(
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

### 示例 3: 图片 URL（OpenAI）

```rust
let message = Message::new(
    Role::User,
    vec![
        ContentPart::text("Describe this image"),
        ContentPart::image_url("https://example.com/image.jpg"),
    ],
);
```

### 示例 4: 多张图片

```rust
let message = Message::new(
    Role::User,
    vec![
        ContentPart::text("Compare these images"),
        ContentPart::image_url("https://example.com/image1.jpg"),
        ContentPart::image_url("https://example.com/image2.jpg"),
    ],
);
```

---

## 🔄 迁移指南

### 旧代码

```rust
let message = Message {
    role: Role::User,
    content: "Hello".to_string(),
    ..Default::default()
};
```

### 新代码

```rust
// 方式 1: 使用构造函数（推荐）
let message = Message::text(Role::User, "Hello");

// 方式 2: 直接构造
let message = Message {
    role: Role::User,
    content: vec![ContentPart::text("Hello")],
    ..Default::default()
};
```

**迁移成本**: 低
- 只需将 `content: "..."` 改为 `content: vec![ContentPart::text("...")]`
- 或使用 `Message::text()` 构造函数

---

## 📝 序列化示例

### 纯文本

```rust
let message = Message::text(Role::User, "Hello");
```

**序列化为**:
```json
{
  "role": "user",
  "content": [
    {"type": "text", "text": "Hello"}
  ]
}
```

### 多模态（Anthropic）

```rust
let message = Message::new(
    Role::User,
    vec![
        ContentPart::text("What's this?"),
        ContentPart::image_base64("image/jpeg", "..."),
    ],
);
```

**序列化为**:
```json
{
  "role": "user",
  "content": [
    {"type": "text", "text": "What's this?"},
    {"type": "image", "source": {"type": "base64", "media_type": "image/jpeg", "data": "..."}}
  ]
}
```

### 多模态（OpenAI）

```rust
let message = Message::new(
    Role::User,
    vec![
        ContentPart::text("Describe this"),
        ContentPart::image_url("https://..."),
    ],
);
```

**序列化为**:
```json
{
  "role": "user",
  "content": [
    {"type": "text", "text": "Describe this"},
    {"type": "image_url", "image_url": {"url": "https://..."}}
  ]
}
```

---

## 🎯 优势总结

1. **简洁** ✅
   - 无需额外的 `MessageContent` 枚举
   - 代码量减少 ~50%
   - 结构清晰

2. **Native** ✅
   - 直接对齐 API 格式
   - 无额外抽象
   - 序列化直接

3. **灵活** ✅
   - 统一处理纯文本和多模态
   - 易于扩展新类型
   - 类型安全

4. **易用** ✅
   - 便捷构造函数
   - 清晰的 API
   - 符合直觉

---

## 📋 实现清单

- [ ] 创建 `src/types/content.rs`
- [ ] 定义 `ContentPart` 枚举
- [ ] 定义 `ImageSource`、`ImageUrl`
- [ ] 更新 `Message` 结构
- [ ] 添加便捷构造函数
- [ ] 更新所有 providers
- [ ] 添加单元测试
- [ ] 添加示例代码
- [ ] 更新文档

---

## 🎉 总结

**推荐方案**: 直接使用 `Vec<ContentPart>`

**核心优势**:
- 🎯 **简洁**: 无需额外枚举层
- 🎯 **Native**: 直接对齐 API
- 🎯 **灵活**: 统一处理所有场景
- 🎯 **易用**: 清晰的 API

**适合场景**:
- ✅ 项目前期阶段
- ✅ 追求简洁设计
- ✅ 不需要向后兼容
- ✅ 直接对齐主流 API

**不适合场景**:
- ❌ 已有大量现有代码
- ❌ 必须向后兼容
- ❌ 需要区分纯文本和多模态

