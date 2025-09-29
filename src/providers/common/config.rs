use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum ParserType {
    Sse,
    Ndjson,
}

// 更多通用配置将在这里添加