/// Test streaming tool_calls parsing logic
///
/// This test simulates the tool_calls format returned by OpenAI streaming API,
/// verifying that llm-connector correctly handles incremental data

#[cfg(feature = "streaming")]
#[tokio::test]
async fn test_streaming_tool_calls_accumulation() {
    use llm_connector::types::{StreamingResponse, ToolCall};
    use std::collections::HashMap;

    // Simulate actual OpenAI streaming tool_calls format
    let chunk1 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "role": "assistant",
                "tool_calls": [{
                    "index": 0,
                    "id": "call_abc123",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": ""
                    }
                }]
            },
            "finish_reason": null
        }]
    }"#;

    let chunk2 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "function": {
                        "arguments": "{\"loc"
                    }
                }]
            },
            "finish_reason": null
        }]
    }"#;

    let chunk3 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "function": {
                        "arguments": "ation\": \"Beijing"
                    }
                }]
            },
            "finish_reason": null
        }]
    }"#;

    let chunk4 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "function": {
                        "arguments": "\"}"
                    }
                }]
            },
            "finish_reason": null
        }]
    }"#;

    let chunk5 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": "tool_calls"
        }]
    }"#;

    // Accumulate tool_calls
    let chunks = vec![chunk1, chunk2, chunk3, chunk4, chunk5];
    let mut accumulated_calls: HashMap<usize, ToolCall> = HashMap::new();

    println!("\n=== Test streaming tool_calls accumulation ===\n");

    for (i, chunk_str) in chunks.iter().enumerate() {
        let response: StreamingResponse = serde_json::from_str(chunk_str).unwrap();

        println!("Chunk #{}: Parsed successfully", i + 1);

        if let Some(choice) = response.choices.first() {
            if let Some(tool_calls) = &choice.delta.tool_calls {
                for delta_call in tool_calls {
                    let index = delta_call.index.unwrap_or(0);

                    accumulated_calls
                        .entry(index)
                        .and_modify(|existing| existing.merge_delta(delta_call))
                        .or_insert_with(|| delta_call.clone());

                    println!("  Accumulated tool_call[{}]:", index);
                    if let Some(call) = accumulated_calls.get(&index) {
                        println!("    id: {}", call.id);
                        println!("    name: {}", call.function.name);
                        println!("    arguments: {}", call.function.arguments);
                    }
                }
            }
        }
    }

    println!("\n📊 Final result:");
    println!("  - Accumulated tool_calls count: {}", accumulated_calls.len());

    for (index, call) in &accumulated_calls {
        println!("\n  Tool Call [{}]:", index);
        println!("    id: {}", call.id);
        println!("    type: {}", call.call_type);
        println!("    function.name: {}", call.function.name);
        println!("    function.arguments: {}", call.function.arguments);

        // Verify completeness
        assert!(call.is_complete(), "Tool call should be complete");
        assert_eq!(call.id, "call_abc123");
        assert_eq!(call.function.name, "get_weather");
        assert_eq!(call.function.arguments, r#"{"location": "Beijing"}"#);
    }

    println!("\n✅ Test passed: tool_calls correctly accumulated");
}

#[cfg(feature = "streaming")]
#[tokio::test]
async fn test_streaming_tool_calls_parsing() {
    use llm_connector::types::StreamingResponse;
    
    // Simulate actual OpenAI streaming tool_calls format
    // According to OpenAI docs, tool_calls in streaming responses are incremental

    // Chunk 1: Start tool_call, contains id, type, function.name
    let chunk1 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "role": "assistant",
                "tool_calls": [{
                    "index": 0,
                    "id": "call_abc123",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": ""
                    }
                }]
            },
            "finish_reason": null
        }]
    }"#;
    
    // Chunk 2: Arguments delta - first part
    let chunk2 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "function": {
                        "arguments": "{\"loc"
                    }
                }]
            },
            "finish_reason": null
        }]
    }"#;
    
    // Chunk 3: Arguments delta - second part
    let chunk3 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "function": {
                        "arguments": "ation\": \"Beijing"
                    }
                }]
            },
            "finish_reason": null
        }]
    }"#;

    // Chunk 4: Arguments delta - complete
    let chunk4 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "function": {
                        "arguments": "\"}"
                    }
                }]
            },
            "finish_reason": null
        }]
    }"#;

    // Chunk 5: End
    let chunk5 = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": "tool_calls"
        }]
    }"#;

    // Parse all chunks
    let chunks = vec![chunk1, chunk2, chunk3, chunk4, chunk5];
    let mut tool_calls_count = 0;
    let mut tool_call_ids = Vec::new();

    println!("\n=== Test streaming tool_calls parsing ===\n");

    for (i, chunk_str) in chunks.iter().enumerate() {
        let response: Result<StreamingResponse, _> = serde_json::from_str(chunk_str);

        match response {
            Ok(chunk) => {
                println!("Chunk #{}: Parsed successfully", i + 1);

                if let Some(choice) = chunk.choices.first() {
                    if let Some(tool_calls) = &choice.delta.tool_calls {
                        tool_calls_count += tool_calls.len();

                        println!("  Found {} tool_calls:", tool_calls.len());
                        for call in tool_calls {
                            println!("    - id: {}", call.id);
                            println!("      name: {}", call.function.name);
                            println!("      arguments: {}", call.function.arguments);
                            
                            if !call.id.is_empty() {
                                tool_call_ids.push(call.id.clone());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Chunk #{}: Parse failed - {}", i + 1, e);
            }
        }
    }

    println!("\n📊 Statistics:");
    println!("  - Total tool_calls occurrences: {}", tool_calls_count);
    println!("  - Unique tool_call IDs: {:?}", tool_call_ids);

    // Verify the issue - this test demonstrates the original problem
    println!("\n⚠️  Original problem demonstration:");
    println!("  - tool_calls appeared in {} chunks", tool_calls_count);
    println!("  - If upstream application uses each chunk's tool_calls directly, it will cause duplicate execution!");
    println!("\nNote: This test demonstrates the pre-fix problem (incremental chunks can be parsed but cause duplicates)");
}

