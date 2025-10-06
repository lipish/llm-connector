//! Simple example showing the difference between supported_models() and fetch_models()
//!
//! Run with: cargo run --example fetch_models_simple

use llm_connector::LlmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Comparing supported_models() vs fetch_models()\n");

    // Example 1: OpenAI
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 1: OpenAI Protocol");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let openai_client = LlmClient::openai("test-key");

    println!("📦 supported_models() - Static/Cached:");
    let static_models = openai_client.supported_models();
    println!("   Returns: {:?}", static_models);
    println!("   ℹ️  This is fast but returns empty for OpenAI\n");

    println!("🌐 fetch_models() - Online from API:");
    println!("   ℹ️  This makes an API call to get real-time model list");
    println!("   ⚠️  Requires valid API key\n");

    // Example 2: DeepSeek (if you have a key)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 2: DeepSeek (OpenAI-compatible)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

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
                let client = LlmClient::openai_compatible(
                    &deepseek.api_key,
                    &deepseek.base_url,
                );

                println!("📦 supported_models():");
                let static_models = client.supported_models();
                println!("   Returns: {:?}", static_models);
                println!("   ✅ Empty - no hardcoded models\n");

                println!("🌐 fetch_models():");
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

    println!("supported_models():");
    println!("  ✅ Fast - no API call");
    println!("  ✅ No authentication needed");
    println!("  ❌ Returns empty for OpenAI protocol");
    println!("  ❌ May be outdated for other protocols\n");

    println!("fetch_models():");
    println!("  ✅ Real-time data from API");
    println!("  ✅ Always up-to-date");
    println!("  ✅ Works with OpenAI-compatible providers");
    println!("  ❌ Requires API call (slower)");
    println!("  ❌ Requires valid API key");
    println!("  ❌ Not supported by all protocols\n");

    println!("💡 Recommendation:");
    println!("  - Use fetch_models() when you need the latest model list");
    println!("  - Use supported_models() for quick checks (if available)");
    println!("  - Cache fetch_models() results to avoid repeated API calls\n");

    Ok(())
}

