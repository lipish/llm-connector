//! Google Gemini Streaming Example
//!
//! Demonstrates how to use Google Gemini provider for streaming chat.
//!
//! Run:
//! ```bash
//! export GEMINI_API_KEY="your-api-key"
//! cargo run --example google_streaming --features streaming
//! ```

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[cfg(feature = "streaming")]
use std::time::Instant;

#[cfg(feature = "streaming")]
use std::time::Duration;

#[cfg(feature = "streaming")]
use std::io::Write;

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("🤖 Google Gemini Streaming Chat Example\n");
    println!("This example requires the `streaming` feature.");
    println!("Run: cargo run --example google_streaming --features streaming");
}

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Google Gemini Streaming Chat Example\n");

    let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        println!("❌ Please set environment variable GEMINI_API_KEY");
        println!("   export GEMINI_API_KEY=your-api-key");
        std::process::exit(1);
    });

    let client = LlmClient::google(&api_key)?;

    let model = match std::env::var("GEMINI_MODEL") {
        Ok(m) if !m.trim().is_empty() => m,
        _ => {
            let models = client.models().await.unwrap_or_default();
            let preferred = [
                "gemini-2.5-flash",
                "gemini-2.0-flash",
                "gemini-1.5-flash",
                "gemini-1.5-flash-latest",
            ];

            preferred
                .iter()
                .find(|m| models.iter().any(|x| x == *m))
                .map(|s| s.to_string())
                .or_else(|| models.into_iter().next())
                .unwrap_or_else(|| "gemini-2.5-flash".to_string())
        }
    };

    println!("Using model: {}", model);

    let prompt = std::env::var("GEMINI_PROMPT").unwrap_or_else(|_| {
        "Write a detailed technical article (at least 1200 words) about how ocean currents work. \
Use Markdown headings, include a short bullet list, and add a brief conclusion."
            .to_string()
    });

    let max_tokens: Option<u32> = std::env::var("GEMINI_MAX_TOKENS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .or(Some(2048));

    let request = ChatRequest {
        model,
        messages: vec![Message::user(prompt)],
        stream: Some(true),
        max_tokens,
        ..Default::default()
    };

    print!("Response: ");
    std::io::stdout().flush()?;

    let mut stream = client.chat_stream(&request).await?;

    let start = Instant::now();
    let mut chunk_idx: u64 = 0;
    let mut timeout_cnt: u64 = 0;

    let per_chunk_timeout = std::env::var("GEMINI_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30);

    loop {
        let next =
            tokio::time::timeout(Duration::from_secs(per_chunk_timeout), stream.next()).await;
        match next {
            Ok(Some(chunk_result)) => {
                chunk_idx += 1;
                let chunk = chunk_result?;

                if let Some(content) = chunk.get_content() {
                    if !content.is_empty() {
                        eprintln!(
                            "[DEBUG] chunk #{}, +{}ms, content_len={}",
                            chunk_idx,
                            start.elapsed().as_millis(),
                            content.len()
                        );
                        print!("{}", content);
                        std::io::stdout().flush()?;
                    }
                } else {
                    // Useful when the provider sends metadata-only chunks.
                    if let Some(choice) = chunk.choices.first() {
                        eprintln!(
                            "\n[DEBUG] chunk #{}, +{}ms, no content (finish_reason={:?}, delta={:?})",
                            chunk_idx,
                            start.elapsed().as_millis(),
                            choice.finish_reason,
                            choice.delta
                        );
                    } else {
                        eprintln!(
                            "\n[DEBUG] chunk #{}, +{}ms, no choices in chunk: {:?}",
                            chunk_idx,
                            start.elapsed().as_millis(),
                            chunk
                        );
                    }
                }
            }
            Ok(None) => {
                eprintln!(
                    "\n[DEBUG] stream ended after {} chunks, +{}ms, timeouts={} ",
                    chunk_idx,
                    start.elapsed().as_millis(),
                    timeout_cnt
                );
                break;
            }
            Err(_) => {
                timeout_cnt += 1;
                eprintln!(
                    "\n[DEBUG] waiting for stream chunk... ({}s timeout, count={})",
                    per_chunk_timeout, timeout_cnt
                );
            }
        }
    }

    println!();
    Ok(())
}
