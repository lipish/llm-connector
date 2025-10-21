//! 消息内容块定义
//!
//! 支持多模态内容，包括文本、图片等

use serde::{Deserialize, Serialize};

/// 消息内容块
///
/// 一条消息可以包含多个内容块，支持文本、图片等多模态内容
///
/// # 示例
///
/// ```rust
/// use llm_connector::types::MessageBlock;
///
/// // 文本块
/// let text = MessageBlock::text("Hello, world!");
///
/// // 图片块（Base64）
/// let image = MessageBlock::image_base64("image/jpeg", "base64_data...");
///
/// // 图片块（URL）
/// let image_url = MessageBlock::image_url("https://example.com/image.jpg");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageBlock {
    /// 文本块
    Text {
        text: String,
    },

    /// 图片块（Anthropic 格式）
    Image {
        source: ImageSource,
    },

    /// 图片 URL 块（OpenAI 格式）
    ImageUrl {
        image_url: ImageUrl,
    },
}

impl MessageBlock {
    /// 创建文本块
    ///
    /// # 示例
    ///
    /// ```rust
    /// use llm_connector::types::MessageBlock;
    ///
    /// let block = MessageBlock::text("Hello, world!");
    /// ```
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// 创建 Base64 图片块（Anthropic 格式）
    ///
    /// # 参数
    ///
    /// - `media_type`: 媒体类型，如 "image/jpeg", "image/png"
    /// - `data`: Base64 编码的图片数据
    ///
    /// # 示例
    ///
    /// ```rust
    /// use llm_connector::types::MessageBlock;
    ///
    /// let block = MessageBlock::image_base64(
    ///     "image/jpeg",
    ///     "iVBORw0KGgoAAAANSUhEUgA..."
    /// );
    /// ```
    pub fn image_base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Image {
            source: ImageSource::Base64 {
                media_type: media_type.into(),
                data: data.into(),
            },
        }
    }

    /// 创建图片 URL 块（Anthropic 格式）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use llm_connector::types::MessageBlock;
    ///
    /// let block = MessageBlock::image_url_anthropic("https://example.com/image.jpg");
    /// ```
    pub fn image_url_anthropic(url: impl Into<String>) -> Self {
        Self::Image {
            source: ImageSource::Url {
                url: url.into(),
            },
        }
    }

    /// 创建图片 URL 块（OpenAI 格式）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use llm_connector::types::MessageBlock;
    ///
    /// let block = MessageBlock::image_url("https://example.com/image.jpg");
    /// ```
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: None,
            },
        }
    }

    /// 创建图片 URL 块（OpenAI 格式，带 detail 参数）
    ///
    /// # 参数
    ///
    /// - `url`: 图片 URL
    /// - `detail`: 图片细节级别，可选值: "auto", "low", "high"
    ///
    /// # 示例
    ///
    /// ```rust
    /// use llm_connector::types::MessageBlock;
    ///
    /// let block = MessageBlock::image_url_with_detail(
    ///     "https://example.com/image.jpg",
    ///     "high"
    /// );
    /// ```
    pub fn image_url_with_detail(url: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: Some(detail.into()),
            },
        }
    }

    /// 获取文本内容（如果是文本块）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use llm_connector::types::MessageBlock;
    ///
    /// let block = MessageBlock::text("Hello");
    /// assert_eq!(block.as_text(), Some("Hello"));
    ///
    /// let image = MessageBlock::image_url("https://...");
    /// assert_eq!(image.as_text(), None);
    /// ```
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// 判断是否为文本块
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// 判断是否为图片块
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image { .. } | Self::ImageUrl { .. })
    }
}

/// 图片来源（Anthropic 格式）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64 编码的图片
    Base64 {
        /// 媒体类型，如 "image/jpeg", "image/png"
        media_type: String,
        /// Base64 编码的图片数据
        data: String,
    },

    /// 图片 URL
    Url {
        /// 图片 URL
        url: String,
    },
}

/// 图片 URL（OpenAI 格式）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUrl {
    /// 图片 URL
    pub url: String,

    /// 图片细节级别
    ///
    /// 可选值: "auto", "low", "high"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_block() {
        let block = MessageBlock::text("Hello");
        assert!(block.is_text());
        assert!(!block.is_image());
        assert_eq!(block.as_text(), Some("Hello"));
    }

    #[test]
    fn test_image_base64_block() {
        let block = MessageBlock::image_base64("image/jpeg", "base64data");
        assert!(!block.is_text());
        assert!(block.is_image());
        assert_eq!(block.as_text(), None);

        match block {
            MessageBlock::Image { source } => match source {
                ImageSource::Base64 { media_type, data } => {
                    assert_eq!(media_type, "image/jpeg");
                    assert_eq!(data, "base64data");
                }
                _ => panic!("Expected Base64 source"),
            },
            _ => panic!("Expected Image block"),
        }
    }

    #[test]
    fn test_image_url_block() {
        let block = MessageBlock::image_url("https://example.com/image.jpg");
        assert!(!block.is_text());
        assert!(block.is_image());

        match block {
            MessageBlock::ImageUrl { image_url } => {
                assert_eq!(image_url.url, "https://example.com/image.jpg");
                assert_eq!(image_url.detail, None);
            }
            _ => panic!("Expected ImageUrl block"),
        }
    }

    #[test]
    fn test_image_url_with_detail() {
        let block = MessageBlock::image_url_with_detail("https://example.com/image.jpg", "high");

        match block {
            MessageBlock::ImageUrl { image_url } => {
                assert_eq!(image_url.url, "https://example.com/image.jpg");
                assert_eq!(image_url.detail, Some("high".to_string()));
            }
            _ => panic!("Expected ImageUrl block"),
        }
    }

    #[test]
    fn test_serialize_text_block() {
        let block = MessageBlock::text("Hello");
        let json = serde_json::to_string(&block).unwrap();
        assert_eq!(json, r#"{"type":"text","text":"Hello"}"#);
    }

    #[test]
    fn test_serialize_image_base64() {
        let block = MessageBlock::image_base64("image/jpeg", "data");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"image""#));
        assert!(json.contains(r#""type":"base64""#));
        assert!(json.contains(r#""media_type":"image/jpeg""#));
    }

    #[test]
    fn test_serialize_image_url() {
        let block = MessageBlock::image_url("https://example.com/image.jpg");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"image_url""#));
        assert!(json.contains(r#""url":"https://example.com/image.jpg""#));
    }

    #[test]
    fn test_deserialize_text_block() {
        let json = r#"{"type":"text","text":"Hello"}"#;
        let block: MessageBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block, MessageBlock::text("Hello"));
    }
}

