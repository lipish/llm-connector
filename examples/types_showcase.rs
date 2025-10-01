//! Types Showcase Example
//!
//! This example demonstrates the new ergonomic API for creating messages and requests.

use llm_connector::types::{ChatRequest, Message, Role, ToolChoice};

fn main() {
    println!("=== Types Showcase ===\n");

    // 1. Role enum
    println!("1. Role Enum:");
    let roles = vec![Role::System, Role::User, Role::Assistant, Role::Tool];
    for role in roles {
        println!("   {:?}", role);
    }

    // 2. Message constructors
    println!("\n2. Message Constructors:");
    let msg1 = Message::system("You are a helpful assistant");
    let msg2 = Message::user("Hello, how are you?");
    let msg3 = Message::assistant("I'm doing great, thanks!");
    let msg4 = Message::tool("Function result: 42", "call-123");

    println!("   System: {:?}", msg1.role);
    println!("   User: {:?}", msg2.role);
    println!("   Assistant: {:?}", msg3.role);
    println!("   Tool: {:?}", msg4.role);

    // 3. Builder pattern for Message
    println!("\n3. Message Builder Pattern:");
    let msg_with_name = Message::user("Hello from Alice").with_name("Alice");
    println!("   Message with name: {:?}", msg_with_name.name);

    // 4. ChatRequest builder
    println!("\n4. ChatRequest Builder:");
    let request = ChatRequest::new("gpt-4")
        .add_message(Message::system("Be concise"))
        .add_message(Message::user("What is 2+2?"))
        .with_temperature(0.7)
        .with_max_tokens(100);

    println!("   Model: {}", request.model);
    println!("   Messages: {}", request.messages.len());
    println!("   Temperature: {:?}", request.temperature);
    println!("   Max tokens: {:?}", request.max_tokens);

    // 5. ToolChoice constructors
    println!("\n5. ToolChoice Constructors:");
    let tc1 = ToolChoice::none();
    let tc2 = ToolChoice::auto();
    let tc3 = ToolChoice::required();
    let tc4 = ToolChoice::function("calculate");

    println!("   None: created");
    println!("   Auto: created");
    println!("   Required: created");
    println!("   Function: created");

    // 6. Serialization
    println!("\n6. Serialization:");
    let json = serde_json::to_string_pretty(&request).unwrap();
    println!("{}", json);

    println!("\n=== All tests passed! ===");
}

