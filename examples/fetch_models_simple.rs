//! Simple example showing how to fetch available models from the API
//!
//! Run with: cargo run --example fetch_models_simple

use llm_connector::LlmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Fetching Available Models from API\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example: OpenAI Protocol");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("🌐 fetch_models() - Fetch models from API:");
    println!("   ℹ️  This makes an API call to get real-time model list");
    println!("   ⚠️  Requires valid API key\n");

    // Try to load from keys.yaml if available
    if let Ok(keys_content) = std::fs::read_to_string("keys.yaml") {
        use serde::{Deserialize};

        #[derive(Debug, Deserialize)]
        struct ProviderConfig {
            api_key: String,
            base_url: String,
        }

        #[derive(Debug, Deserialize)]
        struct KeysConfig {
            providers: std::collections::HashMap<String, ProviderConfig>,
        }

        if let Ok(config) = serde_yaml::from_str::<KeysConfig>(&keys_content) {
            if let Some(deepseek) = config.providers.get("deepseek") {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("Testing with DeepSeek (OpenAI-compatible)");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

                let client = LlmClient::openai(
                    &deepseek.api_key,
                    Some(&deepseek.base_url),
                );

                match client.fetch_models().await {
                    Ok(models) => {
                        println!("   ✅ Success! Found {} models", models.len());
                        println!("   Models: {:?}", models);
                        println!("   ✅ Real-time data from DeepSeek API");
                    }
                    Err(e) => {
                        println!("   ❌ Error: {}", e);
                    }
                }
            }
        }
    } else {
        println!("   ℹ️  keys.yaml not found - skipping live test");
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📝 Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("fetch_models():");
    println!("  ✅ Real-time data from API");
    println!("  ✅ Always up-to-date");
    println!("  ✅ Works with OpenAI-compatible providers");
    println!("  ❌ Requires API call (slower)");
    println!("  ❌ Requires valid API key");
    println!("  ❌ Not supported by all protocols (Anthropic, Aliyun, Ollama)\n");

    println!("💡 Recommendation:");
    println!("  - Use fetch_models() to get the latest model list from the API");
    println!("  - Cache fetch_models() results to avoid repeated API calls");
    println!("  - Supported by: OpenAI and OpenAI-compatible providers (DeepSeek, Zhipu, Moonshot, etc.)\n");

    Ok(())
}

