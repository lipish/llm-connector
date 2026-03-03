# Examples

All examples live in the [`examples/`](https://github.com/lipish/llm-connector/tree/main/examples) directory.

## Running

```bash
# Copy .env.example to .env and fill in your keys
cargo run --example openai
cargo run --example anthropic
cargo run --example google
cargo run --example ollama
cargo run --example deepseek
cargo run --example moonshot
cargo run --example moonshot_tools
cargo run --example moonshot_thinking
cargo run --example zhipu
cargo run --example zhipu_tools
cargo run --example zhipu_thinking
cargo run --example zhipu_vision
cargo run --example aliyun
cargo run --example tencent         # Requires `--features tencent`
cargo run --example multi_modal
cargo run --example tool_calling
cargo run --example google_tools_thinking
cargo run --example minimax
```

## Per-Request Overrides

API key, base URL, and headers can be overridden **per request** without creating a new client.
See [Architecture → Per-Request Overrides](/guide/architecture#per-request-overrides-multi-tenant--gateway).

## Connectivity Test

Run all configured providers at once:

```bash
cargo run --example real_world_connectivity_test
```
