# Examples 和 Tests 清理计划

## 当前状态

- **Examples**: 39 个文件
- **Tests**: 18 个文件（包括 shell 脚本）

---

## 分类分析

### 1. 基础示例（保留）

#### Provider 基础使用
- ✅ `openai_basic.rs` - OpenAI 基础示例
- ✅ `anthropic_streaming.rs` - Anthropic 流式示例
- ✅ `aliyun_basic.rs` - Aliyun 基础示例
- ✅ `zhipu_basic.rs` - Zhipu 基础示例
- ✅ `ollama_basic.rs` - Ollama 基础示例
- ✅ `tencent_basic.rs` - Tencent 基础示例

#### 特殊功能
- ✅ `multimodal_basic.rs` - 多模态示例（v0.5.0 新功能）
- ✅ `ollama_model_management.rs` - Ollama 模型管理
- ✅ `ollama_streaming.rs` - Ollama 流式示例

**保留原因**: 这些是核心功能的基础示例，用户需要参考。

---

### 2. 重复的测试示例（删除）

#### 与基础示例重复
- ❌ `test_aliyun_basic.rs` - 与 `aliyun_basic.rs` 重复
- ❌ `test_deepseek.rs` - 与 `openai_basic.rs` 类似（DeepSeek 使用 OpenAI 协议）
- ❌ `test_moonshot.rs` - 与 `openai_basic.rs` 类似（Moonshot 使用 OpenAI 协议）
- ❌ `test_tencent.rs` - 与 `tencent_basic.rs` 重复
- ❌ `test_volcengine.rs` - 与基础示例类似
- ❌ `test_longcat_openai.rs` - 与 OpenAI 示例类似
- ❌ `test_longcat_anthropic.rs` - 与 Anthropic 示例类似

#### 与流式示例重复
- ❌ `test_aliyun_streaming.rs` - 与 `aliyun_basic.rs` + streaming 重复
- ❌ `zhipu_streaming.rs` - 与 `zhipu_basic.rs` + streaming 重复

**删除原因**: 这些文件只是简单的 API 调用测试，与基础示例重复。

---

### 3. 调试文件（删除）

- ❌ `debug_aliyun_response.rs` - 调试用，不是示例
- ❌ `debug_longcat_stream.rs` - 调试用，不是示例
- ❌ `debug_longcat_anthropic_stream.rs` - 调试用，不是示例
- ❌ `debug_zhipu_streaming_tool_calls.rs` - 调试用，不是示例

**删除原因**: 这些是临时调试文件，不应该作为示例。

---

### 4. 验证文件（删除或移到 tests）

- ❌ `verify_aliyun_choices.rs` - 验证测试，应该在 tests 中
- ❌ `verify_reasoning_content.rs` - 验证测试，应该在 tests 中
- ❌ `verify_tool_fix.rs` - 验证测试，应该在 tests 中

**删除原因**: 这些是测试验证，不是用户示例。

---

### 5. 特定问题测试（删除）

- ❌ `test_zhipu_tool_messages_detailed.rs` - 详细调试，不是示例
- ❌ `test_zhipu_tool_streaming_issue.rs` - 问题调试，不是示例
- ❌ `test_zhipu_flash_streaming_tool_calls.rs` - 特定问题测试
- ❌ `test_zhipu_reasoning_stream.rs` - 特定功能测试

**删除原因**: 这些是针对特定问题的调试文件，不适合作为示例。

---

### 6. 工具调用示例（保留部分）

#### 保留
- ✅ `zhipu_tools.rs` - Zhipu 工具调用基础示例
- ✅ `zhipu_multiround_tools.rs` - 多轮工具调用示例

#### 删除
- ❌ `zhipu_tools_edge_cases.rs` - 边缘情况测试，不是示例
- ❌ `zhipu_tools_streaming.rs` - 与 `zhipu_tools.rs` + streaming 重复
- ❌ `test_deepseek_tools.rs` - 与 OpenAI 工具调用类似
- ❌ `test_openai_tool_streaming.rs` - 与基础工具调用重复
- ❌ `test_glm_models_tool_streaming.rs` - 与 Zhipu 工具调用重复
- ❌ `test_all_providers_tool_streaming.rs` - 测试文件，不是示例

**删除原因**: 工具调用的核心概念已经在保留的示例中展示。

---

### 7. 特殊功能示例（保留）

- ✅ `test_aliyun_enable_thinking.rs` - Aliyun thinking 功能示例（重命名为 `aliyun_thinking.rs`）

**保留原因**: 展示特殊功能。

---

### 8. Shell 测试脚本（删除）

- ❌ `test_aliyun_response_format.sh`
- ❌ `test_aliyun_streaming_format.sh`
- ❌ `test_longcat_anthropic_raw.sh`
- ❌ `test_longcat_anthropic_streaming_raw.sh`
- ❌ `test_tencent_raw.sh`
- ❌ `test_tencent_streaming_raw.sh`
- ❌ `test_volcengine_raw.sh`
- ❌ `test_volcengine_streaming_raw.sh`
- ❌ `test_zhipu_streaming_direct.sh`

**删除原因**: 这些是原始 API 测试脚本，不是 Rust 示例。

---

## 清理计划

### 保留的 Examples (12 个)

#### 基础示例 (6 个)
1. `openai_basic.rs` - OpenAI 基础
2. `anthropic_streaming.rs` - Anthropic 流式
3. `aliyun_basic.rs` - Aliyun 基础
4. `zhipu_basic.rs` - Zhipu 基础
5. `ollama_basic.rs` - Ollama 基础
6. `tencent_basic.rs` - Tencent 基础

#### 特殊功能 (4 个)
7. `multimodal_basic.rs` - 多模态内容
8. `ollama_model_management.rs` - Ollama 模型管理
9. `ollama_streaming.rs` - Ollama 流式
10. `aliyun_thinking.rs` - Aliyun thinking 功能（重命名）

#### 工具调用 (2 个)
11. `zhipu_tools.rs` - 工具调用基础
12. `zhipu_multiround_tools.rs` - 多轮工具调用

### 删除的 Examples (27 个)

#### 重复示例 (9 个)
1. `test_aliyun_basic.rs`
2. `test_deepseek.rs`
3. `test_moonshot.rs`
4. `test_tencent.rs`
5. `test_volcengine.rs`
6. `test_longcat_openai.rs`
7. `test_longcat_anthropic.rs`
8. `test_aliyun_streaming.rs`
9. `zhipu_streaming.rs`

#### 调试文件 (4 个)
10. `debug_aliyun_response.rs`
11. `debug_longcat_stream.rs`
12. `debug_longcat_anthropic_stream.rs`
13. `debug_zhipu_streaming_tool_calls.rs`

#### 验证文件 (3 个)
14. `verify_aliyun_choices.rs`
15. `verify_reasoning_content.rs`
16. `verify_tool_fix.rs`

#### 特定问题测试 (4 个)
17. `test_zhipu_tool_messages_detailed.rs`
18. `test_zhipu_tool_streaming_issue.rs`
19. `test_zhipu_flash_streaming_tool_calls.rs`
20. `test_zhipu_reasoning_stream.rs`

#### 工具调用重复 (7 个)
21. `zhipu_tools_edge_cases.rs`
22. `zhipu_tools_streaming.rs`
23. `test_deepseek_tools.rs`
24. `test_openai_tool_streaming.rs`
25. `test_glm_models_tool_streaming.rs`
26. `test_all_providers_tool_streaming.rs`

### 删除的 Tests (9 个 shell 脚本)

1. `test_aliyun_response_format.sh`
2. `test_aliyun_streaming_format.sh`
3. `test_longcat_anthropic_raw.sh`
4. `test_longcat_anthropic_streaming_raw.sh`
5. `test_tencent_raw.sh`
6. `test_tencent_streaming_raw.sh`
7. `test_volcengine_raw.sh`
8. `test_volcengine_streaming_raw.sh`
9. `test_zhipu_streaming_direct.sh`

---

## 清理后的结构

### Examples (12 个)
```
examples/
├── README.md
├── openai_basic.rs
├── anthropic_streaming.rs
├── aliyun_basic.rs
├── aliyun_thinking.rs
├── zhipu_basic.rs
├── zhipu_tools.rs
├── zhipu_multiround_tools.rs
├── ollama_basic.rs
├── ollama_streaming.rs
├── ollama_model_management.rs
├── tencent_basic.rs
└── multimodal_basic.rs
```

### Tests (9 个 Rust 文件)
```
tests/
├── common/
├── architecture_tests.rs
├── client_tests.rs
├── error_tests.rs
├── openai_protocol_tests.rs
├── protocol_provider_tests.rs
├── protocol_tests.rs
├── streaming_integration_tests.rs
└── types_tests.rs
```

---

## 优势

### 清理前
- 39 个 examples
- 18 个 tests
- 大量重复和调试文件
- 难以维护

### 清理后
- 12 个 examples（减少 69%）
- 9 个 tests（减少 50%）
- 清晰的结构
- 易于维护

### 好处
1. ✅ **减少维护成本** - 文件数量减少 69%
2. ✅ **清晰的示例** - 每个示例都有明确的目的
3. ✅ **避免混淆** - 不再有重复和调试文件
4. ✅ **更快的编译** - 减少编译目标
5. ✅ **更好的文档** - 示例更聚焦

---

## 执行计划

1. ✅ 重命名 `test_aliyun_enable_thinking.rs` → `aliyun_thinking.rs`
2. ✅ 删除 27 个重复/调试/测试示例文件
3. ✅ 删除 9 个 shell 测试脚本
4. ✅ 更新 `examples/README.md`
5. ✅ 验证编译和测试
6. ✅ 提交更改

