//! Aliyun enable_thinking Parameter Test Example
//!
//! Tests the enable_thinking parameter for Aliyun hybrid reasoning mode.

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read API key from environment variables
    let api_key = std::env::var("ALIYUN_API_KEY")
        .expect("ALIYUN_API_KEY environment variable not set");

    println!("🧪 Testing Aliyun enable_thinking parameter");
    println!("{}", "=".repeat(80));

    // Create client
    let client = LlmClient::aliyun(&api_key)?;

    println!("\n📝 Test 1: Hybrid reasoning model + explicitly enabled");
    println!("{}", "-".repeat(80));
    println!("Model: qwen-plus");
    println!("enable_thinking: Some(true) (explicitly enabled)");
    println!("Expected: returns reasoning_content");

    let request = ChatRequest {
        model: "qwen-plus".to_string(),
        messages: vec![Message::text(Role::User, "Which is larger, 9.11 or 9.9? Please explain your reasoning in detail.")],
        enable_thinking: Some(true),
        max_tokens: Some(500),
        ..Default::default()
    };

    println!("\n📤 Sending request...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ Request succeeded!");
            
            if let Some(reasoning) = response.reasoning_content {
                println!("\n🧠 Reasoning:");
                println!("{}", "-".repeat(80));
                println!("{}", reasoning);
                println!("{}", "-".repeat(80));
                println!("✅ Successfully returned reasoning_content (explicit enable works)");
            } else {
                println!("\n⚠️  reasoning_content was not returned");
                println!("   Possible causes:");
                println!("   1. The model does not support reasoning mode");
                println!("   2. API configuration issue");
            }
            
            println!("\n💡 Final answer:");
            println!("{}", response.content);

            if let Some(usage) = response.usage {
                println!("\n📊 Usage:");
                println!("   prompt_tokens: {}", usage.prompt_tokens);
                println!("   completion_tokens: {}", usage.completion_tokens);
                println!("   total_tokens: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 Test 2: Hybrid reasoning model + not specified (disabled by default)");
    println!("{}", "-".repeat(80));
    println!("Model: qwen-plus");
    println!("enable_thinking: None (not specified)");
    println!("Expected: does not return reasoning_content");

    let request = ChatRequest {
        model: "qwen-plus".to_string(),
        messages: vec![Message::text(Role::User, "If a number squared is 144, what is the number?")],
        max_tokens: Some(500),
        ..Default::default()
    };

    println!("\n📤 Sending request...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ Request succeeded!");

            if response.reasoning_content.is_none() {
                println!("\n✅ Correct: reasoning_content not returned (default disabled)");
            } else {
                println!("\n⚠️  Unexpected: reasoning_content was returned");
            }

            println!("\n💡 Answer:");
            println!("{}", response.content);
        }
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 Test 3: Hybrid reasoning model + explicitly disabled");
    println!("{}", "-".repeat(80));
    println!("Model: qwen-plus");
    println!("enable_thinking: Some(false) (explicitly disabled)");
    println!("Expected: does not return reasoning_content");

    let request = ChatRequest {
        model: "qwen-plus".to_string(),
        messages: vec![Message::text(Role::User, "Hello, please introduce yourself")],
        enable_thinking: Some(false),
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("\n📤 Sending request...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ Request succeeded!");
            
            if response.reasoning_content.is_none() {
                println!("\n✅ Correct: reasoning_content not returned (explicit disable works)");
            } else {
                println!("\n⚠️  Unexpected: reasoning_content was returned");
            }
            
            println!("\n💡 Answer:");
            println!("{}", response.content);
        }
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 Test 4: Pure reasoning model (no config required)");
    println!("{}", "-".repeat(80));
    println!("Model: qwq-plus");
    println!("enable_thinking: None (pure reasoning model enabled by default)");
    println!("Expected: returns reasoning_content");

    let request = ChatRequest {
        model: "qwq-plus".to_string(),
        messages: vec![Message::text(Role::User, "Explain why the sky is blue")],
        max_tokens: Some(500),
        ..Default::default()
    };

    println!("\n📤 Sending request...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ Request succeeded!");
            
            if let Some(reasoning) = response.reasoning_content {
                println!("\n🧠 Reasoning:");
                println!("{}", "-".repeat(80));
                println!("{}...", &reasoning[..reasoning.len().min(200)]);
                println!("{}", "-".repeat(80));
                println!("✅ Successfully returned reasoning_content (pure reasoning model)");
            } else {
                println!("\n⚠️  reasoning_content was not returned");
            }
            
            println!("\n💡 Final answer:");
            println!("{}", response.content);
        }
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 Test 5: Non-reasoning model");
    println!("{}", "-".repeat(80));
    println!("Model: qwen-max");
    println!("enable_thinking: None (non-reasoning model)");
    println!("Expected: does not return reasoning_content");

    let request = ChatRequest {
        model: "qwen-max".to_string(),
        messages: vec![Message::text(Role::User, "Hello")],
        max_tokens: Some(50),
        ..Default::default()
    };

    println!("\n📤 Sending request...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ Request succeeded!");
            
            if response.reasoning_content.is_none() {
                println!("\n✅ Correct: reasoning_content not returned (non-reasoning model)");
            } else {
                println!("\n⚠️  Unexpected: reasoning_content was returned");
            }
            
            println!("\n💡 Answer:");
            println!("{}", response.content);
        }
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            return Err(e.into());
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ Aliyun enable_thinking parameter test completed!");
    println!("{}", "=".repeat(80));

    println!("\n📝 Summary:");
    println!("   1. Hybrid reasoning models (e.g., qwen-plus):");
    println!("      - Must explicitly set enable_thinking: Some(true)");
    println!("      - If not set, reasoning is disabled by default");
    println!("   2. Pure reasoning models (e.g., qwq-plus):");
    println!("      - Enabled by default; no configuration required");
    println!("   3. Non-reasoning models (e.g., qwen-max):");
    println!("      - Does not support enable_thinking");
    println!("   4. Unified API:");
    println!("      - response.reasoning_content - Reasoning process");
    println!("      - response.content - Final answer");
    println!("   5. Explicit control:");
    println!("      - User has full control over whether to enable reasoning mode");
    println!("      - No automatic detection; behavior is clear and predictable");

    Ok(())
}
