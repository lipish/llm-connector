use llm_connector::LlmClient;

fn main() {
    println!("Supported LLM Providers:");
    println!("{}", "=".repeat(40));
    
    for (i, provider) in LlmClient::supported_providers().iter().enumerate() {
        println!("{}. {}", i + 1, provider);
    }
    
    println!("{}", "=".repeat(40));
    println!("Total: {} providers", LlmClient::supported_providers().len());
}
