# Rust 编码规则

## 未使用变量规则

### 规则说明

在 Rust 中，如果声明了变量但没有使用，编译器会发出警告。为了消除这些警告，应该使用下划线前缀。

### 规则

**如果变量没有被使用，应该在变量名前加下划线 `_`**

### 示例

#### ❌ 错误写法（会产生警告）

```rust
fn main() {
    let api_key = std::env::var("API_KEY").unwrap();
    // api_key 没有被使用
    
    let client = LlmClient::new(&api_key)?;
    // client 没有被使用
    
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::text(Role::User, "Hello")],
        ..Default::default()
    };
    // request 没有被使用
}
```

**编译器警告**:
```
warning: unused variable: `api_key`
warning: unused variable: `client`
warning: unused variable: `request`
```

#### ✅ 正确写法（无警告）

```rust
fn main() {
    let _api_key = std::env::var("API_KEY").unwrap();
    // 使用 _api_key 表示这个变量故意不使用
    
    let _client = LlmClient::new(&_api_key)?;
    // 使用 _client 表示这个变量故意不使用
    
    let _request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::text(Role::User, "Hello")],
        ..Default::default()
    };
    // 使用 _request 表示这个变量故意不使用
}
```

**无警告！**

### 适用场景

1. **示例代码** - 演示如何创建对象，但不实际使用
2. **调试代码** - 临时创建变量用于调试
3. **条件编译** - 某些条件下不使用的变量
4. **占位代码** - 未完成的功能

### 实际案例

#### 案例 1: debug_longcat_stream.rs

```rust
// ❌ 之前（有警告）
let client = LlmClient::longcat_anthropic(&api_key)?;
let request = ChatRequest { ... };

// ✅ 修复后（无警告）
let _client = LlmClient::longcat_anthropic(&api_key)?;
let _request = ChatRequest { ... };
```

#### 案例 2: verify_reasoning_content.rs

```rust
// ❌ 之前（有警告）
if let Ok(api_key) = std::env::var("ALIYUN_API_KEY") {
    println!("测试 Aliyun...");
    // api_key 没有被使用
}

// ✅ 修复后（无警告）
if let Ok(_api_key) = std::env::var("ALIYUN_API_KEY") {
    println!("测试 Aliyun...");
    // 使用 _api_key 表示这个变量故意不使用
}
```

### 其他选项

#### 1. 使用单个下划线 `_`

如果完全不需要绑定变量：

```rust
// 完全忽略值
if let Ok(_) = std::env::var("API_KEY") {
    println!("API key exists");
}
```

#### 2. 使用 `#[allow(unused_variables)]`

如果有多个未使用变量：

```rust
#[allow(unused_variables)]
fn example() {
    let api_key = "...";
    let client = LlmClient::new(&api_key)?;
    // 不会产生警告
}
```

### 最佳实践

1. **优先使用下划线前缀** - `_variable_name`
   - 保留变量名，便于理解代码意图
   - 明确表示"故意不使用"

2. **使用单个下划线** - `_`
   - 当完全不需要访问值时
   - 通常用于模式匹配

3. **使用 `#[allow]`** - 最后的选择
   - 当有大量未使用变量时
   - 临时调试代码

### 检查命令

```bash
# 检查所有警告
cargo check --all-targets --all-features

# 检查特定文件
cargo check --example debug_longcat_stream

# 使用 clippy 检查
cargo clippy --all-targets --all-features
```

### 自动修复

Rust 编译器会提供修复建议：

```
warning: unused variable: `client`
  --> examples/debug_longcat_stream.rs:13:9
   |
13 |     let client = LlmClient::longcat_anthropic(&api_key)?;
   |         ^^^^^^ help: if this is intentional, prefix it with an underscore: `_client`
```

按照建议修改即可。

---

## 总结

**核心规则**: 未使用的变量应该使用下划线前缀 `_variable_name`

这样可以：
- ✅ 消除编译器警告
- ✅ 明确表达代码意图
- ✅ 保持代码整洁
- ✅ 通过 CI 检查

---

**应用于 llm-connector 项目**:
- 所有示例代码都应遵循此规则
- 所有测试代码都应遵循此规则
- CI 应该检查并拒绝有未使用变量警告的代码

