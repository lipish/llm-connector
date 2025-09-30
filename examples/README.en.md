# llm-connector Examples Documentation

This directory contains various usage examples for the llm-connector library to help you get started quickly and understand how to use different features.

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Environment Variables](#environment-variables)
- [Examples List](#examples-list)
- [Running Examples](#running-examples)
- [FAQ](#faq)

---

## 🚀 Quick Start

### 1. Clone the Project

```bash
git clone https://github.com/lipish/llm-connector.git
cd llm-connector
```

### 2. Configure Environment Variables

Set the API keys for the providers you want to use:

```bash
# DeepSeek
export DEEPSEEK_API_KEY="your-deepseek-api-key"

# Aliyun DashScope (Qwen)
export ALIYUN_API_KEY="your-aliyun-api-key"

# Zhipu AI (GLM)
export ZHIPU_API_KEY="your-zhipu-api-key"

# LongCat (Free Quota Available)
export LONGCAT_API_KEY="your-longcat-api-key"

# Moonshot (Kimi)
export MOONSHOT_API_KEY="your-moonshot-api-key"
```

### 3. Run Examples

```bash
# Run DeepSeek example
cargo run --example deepseek_example

# Run LongCat example
cargo run --example longcat_demo
```

---

## 🔑 Environment Variables

### Supported Environment Variables

| Variable | Provider | How to Get | Required |
|---------|---------|-----------|----------|
| `DEEPSEEK_API_KEY` | DeepSeek | [DeepSeek Platform](https://platform.deepseek.com/) | Optional |
| `ALIYUN_API_KEY` | Aliyun DashScope | [Aliyun Console](https://dashscope.console.aliyun.com/) | Optional |
| `ZHIPU_API_KEY` | Zhipu AI | [Zhipu Platform](https://open.bigmodel.cn/) | Optional |
| `LONGCAT_API_KEY` | LongCat | [LongCat Platform](https://longcat.chat/platform/) | Optional |
| `MOONSHOT_API_KEY` | Moonshot | [Moonshot Platform](https://platform.moonshot.cn/) | Optional |

### Configuration Methods

#### Method 1: Temporary (Current Terminal Session)

```bash
export DEEPSEEK_API_KEY="sk-xxxxxxxxxxxxxxxx"
```

#### Method 2: Permanent (Add to Shell Config)

**Bash (~/.bashrc or ~/.bash_profile):**
```bash
echo 'export DEEPSEEK_API_KEY="sk-xxxxxxxxxxxxxxxx"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh (~/.zshrc):**
```bash
echo 'export DEEPSEEK_API_KEY="sk-xxxxxxxxxxxxxxxx"' >> ~/.zshrc
source ~/.zshrc
```

#### Method 3: Using .env File (Recommended for Development)

Create a `.env` file (Note: Don't commit to Git):

```bash
# .env
DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxxxxx
ALIYUN_API_KEY=sk-xxxxxxxxxxxxxxxx
ZHIPU_API_KEY=xxxxxxxxxxxxxxxx
LONGCAT_API_KEY=ak_xxxxxxxxxxxxxxxx
MOONSHOT_API_KEY=sk-xxxxxxxxxxxxxxxx
```

Then load it manually or use `direnv`:

```bash
# Manual loading
source .env

# Or use direnv (requires installation)
direnv allow
```

---

## 📚 Examples List

### 1. `deepseek_example.rs` - DeepSeek Basic Example

**Purpose:** Demonstrates how to use the DeepSeek provider for basic chat conversations.

**Features:**
- Supports both environment variables and manual configuration
- Shows basic request/response flow
- Includes error handling examples

**Run:**
```bash
export DEEPSEEK_API_KEY="your-api-key"
cargo run --example deepseek_example
```

**Use Cases:**
- First-time users of llm-connector
- Learning basic API call flow
- Understanding DeepSeek provider usage

---

### 2. `longcat_demo.rs` - LongCat Complete Demo

**Purpose:** Demonstrates complete usage of LongCat API, including multiple models and configuration options.

**Features:**
- LongCat provides free daily quota (500,000 tokens)
- Supports multiple models: LongCat-Flash-Chat, LongCat-Flash-Thinking
- Shows factory pattern usage
- Includes detailed configuration instructions

**Run:**
```bash
export LONGCAT_API_KEY="your-api-key"
cargo run --example longcat_demo
```

**Use Cases:**
- Want to use free LLM API
- Learning factory pattern usage
- Understanding LongCat platform features

**Getting API Key:**
1. Visit [LongCat Platform](https://longcat.chat/platform/)
2. Register an account
3. Create a key in the API Keys page
4. Free quota: 500,000 tokens/day (can request increase to 5,000,000)

---

### 3. `protocol_architecture_demo.rs` - Protocol Architecture Demo

**Purpose:** Showcases llm-connector's core architecture design - protocol-based provider organization.

**Features:**
- Demonstrates three protocols: OpenAI, Anthropic, Aliyun
- Shows how to add new providers
- Explains protocol adapter mechanics
- No real API key required (architecture demo only)

**Run:**
```bash
cargo run --example protocol_architecture_demo
```

**Use Cases:**
- Understanding project architecture
- Learning how to extend new providers
- Understanding protocol adapter pattern

---

### 4. `test_all_providers.rs` - Test All Providers

**Purpose:** Batch test all configured providers to verify API connections and functionality.

**Features:**
- Supports multiple providers: DeepSeek, Aliyun, Zhipu, LongCat, Moonshot
- Automatically skips unconfigured providers
- Shows response results for each provider
- Useful for verifying configuration

**Run:**
```bash
# Set API keys for all providers you want to test
export DEEPSEEK_API_KEY="your-deepseek-key"
export ALIYUN_API_KEY="your-aliyun-key"
export ZHIPU_API_KEY="your-zhipu-key"
export LONGCAT_API_KEY="your-longcat-key"
export MOONSHOT_API_KEY="your-moonshot-key"

cargo run --example test_all_providers
```

**Use Cases:**
- Verifying multiple provider configurations
- Batch testing API connections
- Comparing responses from different providers

---

### 5. `verify_real_api_calls.rs` - Verify Real API Calls

**Purpose:** Verifies real API calls through multiple different requests.

**Features:**
- Sends multiple different questions
- Shows detailed request and response information
- Verifies response diversity
- Useful for debugging and verification

**Run:**
```bash
export DEEPSEEK_API_KEY="your-api-key"
cargo run --example verify_real_api_calls
```

**Use Cases:**
- Verifying API calls are real
- Debugging request/response flow
- Testing different prompt effects

---

### 6. `providers.toml` - Configuration File Example

**Purpose:** Shows how to use TOML configuration files to manage multiple providers.

**Features:**
- Unified configuration format
- Supports multiple providers
- Includes descriptions of all configuration options
- Can be used as a template

**Usage:**
```bash
# Copy example configuration
cp examples/providers.toml my-providers.toml

# Edit configuration file with real API keys
vim my-providers.toml

# Load configuration in code
# (requires config feature)
```

---

## 🎯 Running Examples

### Basic Run

```bash
# Run a single example
cargo run --example <example_name>

# For example:
cargo run --example deepseek_example
```

### Run with Environment Variables

```bash
# Method 1: Set before command
DEEPSEEK_API_KEY="your-key" cargo run --example deepseek_example

# Method 2: Export then run
export DEEPSEEK_API_KEY="your-key"
cargo run --example deepseek_example
```

### View Detailed Output

```bash
# Enable log output
RUST_LOG=debug cargo run --example deepseek_example

# Or use trace level
RUST_LOG=trace cargo run --example deepseek_example
```

### Compile All Examples

```bash
# Check if all examples compile
cargo check --examples

# Compile all examples
cargo build --examples

# Compile release version
cargo build --examples --release
```

---

## ❓ FAQ

### 1. How to Get API Keys?

**DeepSeek:**
- Visit [DeepSeek Platform](https://platform.deepseek.com/)
- Register and login
- Create a new key in the API Keys page

**Aliyun DashScope:**
- Visit [Aliyun DashScope](https://dashscope.console.aliyun.com/)
- Enable the service
- Create a key in the API-KEY management page

**Zhipu AI:**
- Visit [Zhipu Platform](https://open.bigmodel.cn/)
- Register and complete identity verification
- Get API Key from personal center

**LongCat:**
- Visit [LongCat Platform](https://longcat.chat/platform/)
- Register an account
- Create a key in the API Keys page
- Free quota: 500,000 tokens/day

### 2. What to Do If Examples Fail?

**Check environment variables:**
```bash
# Check if environment variable is set
echo $DEEPSEEK_API_KEY

# If empty, set it first
export DEEPSEEK_API_KEY="your-api-key"
```

**Verify API key is valid:**
- Confirm key hasn't expired
- Confirm account has sufficient balance or quota
- Confirm key has correct permissions

**View detailed error messages:**
```bash
RUST_LOG=debug cargo run --example deepseek_example
```

### 3. How to Add New Providers?

Refer to the `protocol_architecture_demo.rs` example to learn how to:
1. Choose the appropriate protocol (OpenAI, Anthropic, or Aliyun)
2. Create provider configuration
3. Use factory pattern to create provider instances

### 4. Can I Use Multiple Providers Simultaneously?

Yes! Refer to the `test_all_providers.rs` example, which shows how to:
- Configure multiple providers
- Switch providers at runtime
- Manage multiple API keys

### 5. Can Example Configurations Be Used in Production?

Example code is mainly for learning and testing. For production environments, consider:
- More robust error handling
- Adding logging and monitoring
- Implementing retry mechanisms
- Using configuration files to manage keys
- Adding rate limiting

---

## 📖 More Resources

- [Project Homepage](https://github.com/lipish/llm-connector)
- [API Documentation](../docs/specs/API.md)
- [Configuration Guide](../docs/CONFIGURATION_GUIDE.md)
- [Architecture Design](../docs/ARCHITECTURE_DESIGN.md)

---

## 🤝 Contributing

If you have new example ideas or find issues, feel free to:
- Submit an Issue
- Create a Pull Request
- Share your usage experience

---

## 📄 License

MIT License - See [LICENSE](../LICENSE) file for details

