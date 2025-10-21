# 多模态内容迁移计划

## 📋 当前状态

已完成：
- ✅ 创建 `MessageBlock` 类型定义
- ✅ 更新 `Message.content` 为 `Vec<MessageBlock>`
- ✅ 添加便捷构造函数

待完成：
- ❌ 更新所有 protocols 支持 `Vec<MessageBlock>`
- ❌ 更新所有 providers
- ❌ 更新示例代码
- ❌ 添加测试

---

## 🔧 需要更新的文件

### 1. Protocols

#### OpenAI Protocol (`src/protocols/openai.rs`)

**问题**:
```rust
// 错误：期望 String，实际是 Vec<MessageBlock>
content: msg.content.clone(),
```

**解决方案**:
```rust
// 需要将 Vec<MessageBlock> 转换为 OpenAI 格式
// OpenAI 支持两种格式：
// 1. 纯文本：content: "text"
// 2. 多模态：content: [{"type": "text", "text": "..."}, ...]

fn convert_content_to_openai(blocks: &[MessageBlock]) -> serde_json::Value {
    if blocks.len() == 1 && blocks[0].is_text() {
        // 纯文本：直接返回字符串
        json!(blocks[0].as_text().unwrap())
    } else {
        // 多模态：返回数组
        json!(blocks)
    }
}
```

#### Anthropic Protocol (`src/protocols/anthropic.rs`)

**问题**:
```rust
// 错误：Vec<MessageBlock> 不能直接 format
format!("{}\n\n{}", existing, msg.content)
```

**解决方案**:
```rust
// Anthropic 只支持数组格式
// content: [{"type": "text", "text": "..."}, ...]

fn convert_content_to_anthropic(blocks: &[MessageBlock]) -> Vec<AnthropicContentBlock> {
    blocks.iter().map(|block| {
        match block {
            MessageBlock::Text { text } => {
                AnthropicContentBlock::Text { text: text.clone() }
            }
            MessageBlock::Image { source } => {
                AnthropicContentBlock::Image { source: source.clone() }
            }
            MessageBlock::ImageUrl { image_url } => {
                // 转换为 Anthropic 格式
                AnthropicContentBlock::Image {
                    source: ImageSource::Url { url: image_url.url.clone() }
                }
            }
        }
    }).collect()
}
```

### 2. Providers

需要更新的 providers：
- Aliyun
- Zhipu
- Ollama
- Tencent
- Volcengine
- LongCat
- Moonshot
- DeepSeek

大多数 providers 使用 OpenAI 兼容格式，可以复用 OpenAI 的转换逻辑。

---

## 🎯 实现策略

### 方案 A: 渐进式迁移（推荐）

1. **第一阶段**: 添加转换辅助函数
   - 创建 `MessageBlock` 到各种格式的转换函数
   - 保持现有 API 不变

2. **第二阶段**: 更新 protocols
   - 逐个更新 OpenAI, Anthropic 等
   - 测试每个 protocol

3. **第三阶段**: 更新示例和文档
   - 更新所有示例代码
   - 更新文档

### 方案 B: 一次性迁移

直接更新所有文件，一次性完成迁移。

**推荐**: 方案 A（渐进式）

---

## 📝 转换辅助函数

### 1. MessageBlock 到纯文本

```rust
impl Message {
    /// 提取所有文本内容
    pub fn content_as_text(&self) -> String {
        self.content.iter()
            .filter_map(|block| block.as_text())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

### 2. MessageBlock 到 OpenAI 格式

```rust
// src/protocols/openai.rs

fn convert_message_content(blocks: &[MessageBlock]) -> serde_json::Value {
    if blocks.len() == 1 && blocks[0].is_text() {
        // 纯文本：返回字符串
        json!(blocks[0].as_text().unwrap())
    } else {
        // 多模态：返回数组
        json!(blocks)
    }
}
```

### 3. MessageBlock 到 Anthropic 格式

```rust
// src/protocols/anthropic.rs

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock {
    Text { text: String },
    Image { source: ImageSource },
}

fn convert_to_anthropic_blocks(blocks: &[MessageBlock]) -> Vec<AnthropicContentBlock> {
    blocks.iter().map(|block| {
        match block {
            MessageBlock::Text { text } => {
                AnthropicContentBlock::Text { text: text.clone() }
            }
            MessageBlock::Image { source } => {
                AnthropicContentBlock::Image { source: source.clone() }
            }
            MessageBlock::ImageUrl { image_url } => {
                // OpenAI 格式转 Anthropic 格式
                AnthropicContentBlock::Image {
                    source: ImageSource::Url { url: image_url.url.clone() }
                }
            }
        }
    }).collect()
}
```

---

## 🧪 测试计划

### 1. 单元测试

```rust
#[test]
fn test_message_text() {
    let msg = Message::text(Role::User, "Hello");
    assert_eq!(msg.content.len(), 1);
    assert!(msg.content[0].is_text());
}

#[test]
fn test_message_multimodal() {
    let msg = Message::new(
        Role::User,
        vec![
            MessageBlock::text("What's this?"),
            MessageBlock::image_url("https://..."),
        ],
    );
    assert_eq!(msg.content.len(), 2);
}
```

### 2. 集成测试

测试每个 protocol 的序列化/反序列化：
- OpenAI protocol
- Anthropic protocol
- 其他 providers

---

## 📋 实施清单

### 阶段 1: 核心转换函数
- [ ] 添加 `Message::content_as_text()`
- [ ] 添加 OpenAI 转换函数
- [ ] 添加 Anthropic 转换函数
- [ ] 添加单元测试

### 阶段 2: 更新 Protocols
- [ ] 更新 OpenAI protocol
- [ ] 更新 Anthropic protocol
- [ ] 测试 protocols

### 阶段 3: 更新 Providers
- [ ] 更新 Aliyun
- [ ] 更新 Zhipu
- [ ] 更新 Ollama
- [ ] 更新其他 providers

### 阶段 4: 示例和文档
- [ ] 更新所有示例代码
- [ ] 更新 README
- [ ] 添加多模态使用示例
- [ ] 更新文档

---

## 🎯 预期结果

完成后：
- ✅ 支持多模态内容（文本 + 图片）
- ✅ 统一的 API 接口
- ✅ 类型安全
- ✅ 所有 protocols 正常工作
- ✅ 所有测试通过

---

## ⚠️ 注意事项

1. **向后兼容性**: 
   - 旧代码需要从 `Message::new(role, "text")` 改为 `Message::text(role, "text")`
   - 提供清晰的迁移指南

2. **序列化格式**:
   - OpenAI: 支持 string 或 array
   - Anthropic: 只支持 array
   - 需要正确处理转换

3. **测试覆盖**:
   - 纯文本消息
   - 多模态消息
   - 各种 provider 格式

---

## 🚀 下一步

建议按以下顺序进行：

1. **立即**: 添加转换辅助函数
2. **然后**: 更新 OpenAI protocol
3. **接着**: 更新 Anthropic protocol
4. **最后**: 更新其他 providers 和示例

是否继续实施？

