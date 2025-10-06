//! YAML Configuration Demo
//!
//! This example demonstrates loading provider configuration from a YAML file.
//! 
//! To run this example:
//! 1. Create a config.yaml file (see example below)
//! 2. cargo run --example yaml_config_demo --features yaml

#[cfg(feature = "yaml")]
use llm_connector::config::RegistryConfig;
#[cfg(feature = "yaml")]
use llm_connector::registry::ProviderRegistry;

#[cfg(feature = "yaml")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== YAML Configuration Demo ===\n");

    // Example YAML content
    let yaml_example = r#"
providers:
  deepseek:
    protocol: openai
    api_key: sk-your-deepseek-key
    base_url: https://api.deepseek.com/v1
    timeout_ms: 30000
    
  claude:
    protocol: anthropic
    api_key: sk-ant-your-anthropic-key
    timeout_ms: 60000
    
  qwen:
    protocol: aliyun
    api_key: sk-your-aliyun-key
"#;

    println!("Example YAML configuration:");
    println!("{}", yaml_example);
    println!("\n--- Loading Configuration ---\n");

    // In a real application, you would load from a file:
    // let config = RegistryConfig::from_yaml_file("config.yaml")?;
    
    // For this demo, we'll parse the YAML string directly
    let config: RegistryConfig = serde_yaml::from_str(yaml_example)?;
    
    println!("✅ Configuration loaded successfully!");
    println!("\nProviders found:");
    for name in config.provider_names() {
        if let Some(entry) = config.get_provider(name) {
            println!("  - {} (protocol: {})", name, entry.protocol);
        }
    }

    println!("\n--- Creating Provider Registry ---\n");
    
    let registry = ProviderRegistry::from_config(config)?;
    
    println!("✅ Registry created successfully!");
    println!("\nAvailable providers:");
    for name in registry.provider_names() {
        if let Some(provider) = registry.get_provider(name) {
            println!("  - {}: {:?}", name, provider.supported_models());
        }
    }

    println!("\n=== Demo Complete ===");
    println!("\nTo use in your application:");
    println!("1. Enable the 'yaml' feature in Cargo.toml");
    println!("2. Create a config.yaml file");
    println!("3. Load with: RegistryConfig::from_yaml_file(\"config.yaml\")");

    Ok(())
}

#[cfg(not(feature = "yaml"))]
fn main() {
    println!("This example requires the 'yaml' feature.");
    println!("Run with: cargo run --example yaml_config_demo --features yaml");
}

