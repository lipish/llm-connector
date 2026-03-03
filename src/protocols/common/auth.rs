//! Common Authentication Strategies

/// Create standard Bearer token header
pub fn bearer_auth(api_key: &str) -> Vec<(String, String)> {
    vec![
        ("Authorization".to_string(), format!("Bearer {}", api_key)),
        ("Content-Type".to_string(), "application/json".to_string()),
    ]
}

/// Create standard API Key header (e.g. for Anthropic)
pub fn api_key_header(api_key: &str, header_name: &str) -> Vec<(String, String)> {
    vec![
        (header_name.to_string(), api_key.to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
    ]
}
