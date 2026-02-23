//! Message Content Block Definition
//!
//! Supports multi-modal content including text, images, etc.

use serde::{Deserialize, Serialize};

/// Message Content Block
///
/// A message can contain multiple content blocks, supporting multi-modal content like text, images, etc.
///
/// # Example
///
/// ```rust
/// use llm_connector::types::MessageBlock;
///
/// // Text block
/// let text = MessageBlock::text("Hello, world!");
///
/// // Image block (Base64)
/// let image = MessageBlock::image_base64("image/jpeg", "base64_data...");
///
/// // Image block (URL)
/// let image_url = MessageBlock::image_url("https://example.com/image.jpg");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageBlock {
    /// Text block
    Text { text: String },

    /// Image block (Anthropic format)
    Image { source: ImageSource },

    /// Image URL block (OpenAI format)
    ImageUrl { image_url: ImageUrl },
}

impl MessageBlock {
    /// Create text block
    ///
    /// # Example
    ///
    /// ```rust
    /// use llm_connector::types::MessageBlock;
    ///
    /// let block = MessageBlock::text("Hello, world!");
    /// ```
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create Base64 Image block (Anthropic format)
    ///
    /// # Parameters
    ///
    /// - `media_type`: Media type, such as "image/jpeg", "image/png"
    /// - `data`: Base64 encoded image data
    ///
    /// # Example
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

    /// Create image URL block (Anthropic format)
    ///
    /// # Example
    ///
    /// ```rust
    /// use llm_connector::types::MessageBlock;
    ///
    /// let block = MessageBlock::image_url_anthropic("https://example.com/image.jpg");
    /// ```
    pub fn image_url_anthropic(url: impl Into<String>) -> Self {
        Self::Image {
            source: ImageSource::Url { url: url.into() },
        }
    }

    /// Create image URL block (OpenAI format)
    ///
    /// # Example
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

    /// Create image URL block (OpenAI format, with detail parameter)
    ///
    /// # Parameters
    ///
    /// - `url`: Image URL
    /// - `detail`: Image detail level, optional values: "auto", "low", "high"
    ///
    /// # Example
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

    /// Get text content (if is text block)
    ///
    /// # Example
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

    /// CheckisifasText block
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// Check if is image block
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image { .. } | Self::ImageUrl { .. })
    }
}

/// Image source (Anthropic format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64 encoded image
    Base64 {
        /// Media type, such as "image/jpeg", "image/png"
        media_type: String,
        /// Base64 encoded image data
        data: String,
    },

    /// Image URL
    Url {
        /// Image URL
        url: String,
    },
}

/// Image URL (OpenAI format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUrl {
    /// Image URL
    pub url: String,

    /// Image detail level
    ///
    /// Optional values: "auto", "low", "high"
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
