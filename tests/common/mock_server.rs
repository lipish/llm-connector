//! Mock DeepSeek server to test request format without real API key

use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🧪 Starting Mock DeepSeek Server");
    println!("This server will capture and validate the HTTP requests from llm-connector");

    // Start mock server
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let server_addr = listener.local_addr()?;
    println!("🚀 Mock server listening on http://{}", server_addr);

    // Start the client test in a separate task
    let client_handle = tokio::spawn(async move {
        // Wait a bit for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        println!("\n📡 Starting client test...");
        test_client(server_addr.to_string()).await
    });

    // Handle one connection
    let (mut socket, addr) = listener.accept().await?;
    println!("📥 Received connection from: {}", addr);

    let mut buffer = vec![0; 4096];
    let n = socket.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    println!("\n📋 Captured HTTP Request:");
    println!("{}", request);

    // Parse the request
    let lines: Vec<&str> = request.lines().collect();
    let mut headers = HashMap::new();
    let mut body_start = 0;

    // Parse headers
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
        if line.contains(":") && i > 0 {
            let parts: Vec<&str> = line.splitn(2, ":").collect();
            if parts.len() == 2 {
                headers.insert(parts[0].trim().to_lowercase(), parts[1].trim());
            }
        }
    }

    // Get request body
    let body = if body_start < lines.len() {
        lines[body_start..].join("\n")
    } else {
        String::new()
    };

    println!("\n🔍 Request Analysis:");

    // Check method and path
    if let Some(first_line) = lines.first() {
        println!("Method & Path: {}", first_line);
        if first_line.contains("POST /chat/completions") {
            println!("✅ Correct endpoint: POST /chat/completions");
        } else {
            println!("❌ Wrong endpoint, expected: POST /chat/completions");
        }
    }

    // Check headers
    println!("\n📋 Headers:");
    for (key, value) in &headers {
        println!("  {}: {}", key, value);
    }

    // Validate required headers
    let mut validation_passed = true;

    if let Some(auth) = headers.get("authorization") {
        if auth.starts_with("Bearer ") {
            println!("✅ Authorization header format correct");
        } else {
            println!("❌ Authorization header format incorrect");
            validation_passed = false;
        }
    } else {
        println!("❌ Missing Authorization header");
        validation_passed = false;
    }

    if let Some(content_type) = headers.get("content-type") {
        if content_type.contains("application/json") {
            println!("✅ Content-Type header correct");
        } else {
            println!("❌ Content-Type header incorrect");
            validation_passed = false;
        }
    } else {
        println!("❌ Missing Content-Type header");
        validation_passed = false;
    }

    // Parse and validate JSON body
    println!("\n📄 Request Body:");
    println!("{}", body);

    if !body.trim().is_empty() {
        match serde_json::from_str::<Value>(&body) {
            Ok(json_body) => {
                println!("✅ Valid JSON format");

                // Validate required fields
                if let Some(model) = json_body.get("model") {
                    if model.as_str() == Some("deepseek-chat") {
                        println!("✅ Model field correct: {}", model);
                    } else {
                        println!("⚠️  Model field: {}", model);
                    }
                } else {
                    println!("❌ Missing model field");
                    validation_passed = false;
                }

                if let Some(messages) = json_body.get("messages") {
                    if messages.is_array() {
                        println!("✅ Messages field is array");
                        if let Some(msgs) = messages.as_array() {
                            for (i, msg) in msgs.iter().enumerate() {
                                if let (Some(role), Some(content)) =
                                    (msg.get("role"), msg.get("content"))
                                {
                                    println!("  Message {}: role={}, content={}", i, role, content);
                                } else {
                                    println!("❌ Message {} missing role or content", i);
                                    validation_passed = false;
                                }
                            }
                        }
                    } else {
                        println!("❌ Messages field is not array");
                        validation_passed = false;
                    }
                } else {
                    println!("❌ Missing messages field");
                    validation_passed = false;
                }

                // Check optional fields
                if let Some(temp) = json_body.get("temperature") {
                    println!("✅ Temperature: {}", temp);
                }
                if let Some(max_tokens) = json_body.get("max_tokens") {
                    println!("✅ Max tokens: {}", max_tokens);
                }
                if let Some(stream) = json_body.get("stream") {
                    println!("✅ Stream: {}", stream);
                }
            }
            Err(e) => {
                println!("❌ Invalid JSON: {}", e);
                validation_passed = false;
            }
        }
    } else {
        println!("❌ Empty request body");
        validation_passed = false;
    }

    // Send response
    let response = if validation_passed {
        // Send a mock successful response
        let mock_response = json!({
            "id": "chatcmpl-mock-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "deepseek-chat",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! This is a mock response from the test server. Your request format is correct!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30
            }
        });

        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            mock_response.to_string().len(),
            mock_response
        )
    } else {
        // Send error response
        let error_response = json!({
            "error": {
                "message": "Request validation failed",
                "type": "invalid_request_error"
            }
        });

        format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            error_response.to_string().len(),
            error_response
        )
    };

    socket.write_all(response.as_bytes()).await?;

    println!("\n📊 Validation Summary:");
    if validation_passed {
        println!("🎉 ALL VALIDATIONS PASSED!");
        println!("✅ Request format is correct for DeepSeek API");
        println!("✅ Headers are properly formatted");
        println!("✅ JSON body structure is valid");
        println!("✅ Required fields are present");
    } else {
        println!("❌ Some validations failed");
        println!("💡 Check the errors above for details");
    }

    // Wait for client to finish
    let _ = client_handle.await;

    Ok(())
}

async fn test_client(server_addr: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use llm_connector::{ChatRequest, Client, Config, Message, ProviderConfig};

    // Configure client to use our mock server
    let config = Config {
        deepseek: Some(ProviderConfig {
            api_key: "test-api-key".to_string(),
            base_url: Some(format!("http://{}", server_addr)),
            timeout_ms: Some(5000),
            proxy: None,
        }),
        openai: None,
        zhipu: None,
        ..Default::default()
    };

    let client = Client::with_config(config);

    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
                ..Default::default()
            },
            Message {
                role: "user".to_string(),
                content: "Hello, this is a test message!".to_string(),
                ..Default::default()
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("📤 Sending request to mock server...");

    match client.chat(request).await {
        Ok(response) => {
            println!("✅ Client received successful response!");
            println!("📝 Response ID: {}", response.id);
            println!("🤖 Model: {}", response.model);
            if let Some(choice) = response.choices.first() {
                println!("💬 Content: {}", choice.message.content);
            }
            if let Some(usage) = response.usage {
                println!(
                    "📊 Usage: {} prompt + {} completion = {} total tokens",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                );
            }
        }
        Err(e) => {
            println!("❌ Client error: {}", e);
        }
    }

    Ok(())
}
