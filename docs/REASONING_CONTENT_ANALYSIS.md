# Reasoning Content 支持分析

## 📋 概述

本文档分析各个 LLM Provider 对推理内容（reasoning content / thinking process）的支持情况。

**更新日期**: 2025-10-21

---

## 🎯 什么是 Reasoning Content？

Reasoning Content（推理内容）是指 AI 模型在生成最终答案前的**思考过程**。类似于人类解决问题时的"草稿纸"，展示了模型如何一步步推理得出结论。

**典型应用场景**:
- 数学推理
- 逻辑推理
- 复杂问题求解
- 需要展示思考过程的场景

---

## 📊 各 Provider 支持情况

### 1. OpenAI ✅ 支持

**推理模型**: 
- `o1-preview`
- `o1-mini`
- `o1` (最新)

**字段名称**: `reasoning_content`

**API 响应格式**:
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "最终答案",
      "reasoning_content": "思考过程..."
    }
  }]
}
```

**当前实现状态**: ✅ **已支持**
- OpenAIResponseMessage 包含 reasoning_content 字段
- parse_response 自动提取到 ChatResponse.reasoning_content

**测试建议**: 使用 o1 模型测试

---

### 2. DeepSeek ✅ 支持

**推理模型**: 
- `deepseek-reasoner`

**字段名称**: `reasoning_content`

**API 响应格式**:
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "最终答案",
      "reasoning_content": "思考过程..."
    }
  }]
}
```

**当前实现状态**: ✅ **已支持**
- 使用 OpenAI 兼容格式
- 自动提取 reasoning_content

**测试状态**: ✅ 已验证

---

### 3. Moonshot (Kimi) ✅ 支持

**推理模型**: 
- `kimi-thinking-preview`

**字段名称**: `reasoning_content`

**API 响应格式**:
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "最终答案",
      "reasoning_content": "思考过程..."
    }
  }]
}
```

**官方文档**: https://platform.moonshot.ai/docs/guide/use-kimi-thinking-preview-model

**当前实现状态**: ✅ **已支持**
- 使用 OpenAI 兼容格式
- 自动提取 reasoning_content

**测试建议**: 使用 kimi-thinking-preview 模型测试

---

### 4. Zhipu (智谱) ⚠️ 部分支持

**推理模型**: 
- `glm-z1` (GLM-Zero-Preview)
- `GLM-4.1V-Thinking` (视觉推理)

**字段名称**: 嵌入在 `content` 中

**API 响应格式**:
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "###Thinking\n思考过程...\n###Response\n最终答案"
    }
  }]
}
```

**特点**:
- 推理内容和答案都在 `content` 字段中
- 使用 `###Thinking` 和 `###Response` 标记分隔
- 需要手动解析

**当前实现状态**: ❌ **未支持**
- 当前实现直接返回整个 content
- 未解析 ###Thinking 和 ###Response

**改进建议**: 
1. 检测 content 中是否包含 `###Thinking` 标记
2. 如果包含，解析并分离到 reasoning_content 和 content
3. 如果不包含，保持原样

**实现示例**:
```rust
fn parse_zhipu_content(content: &str) -> (Option<String>, String) {
    if content.contains("###Thinking") && content.contains("###Response") {
        // 分离推理内容和答案
        let parts: Vec<&str> = content.split("###Response").collect();
        if parts.len() == 2 {
            let thinking = parts[0]
                .replace("###Thinking", "")
                .trim()
                .to_string();
            let response = parts[1].trim().to_string();
            return (Some(thinking), response);
        }
    }
    (None, content.to_string())
}
```

---

### 5. Aliyun (通义千问) ✅ 支持

**推理模型**:
- `qwen-plus` (混合推理模式，需启用)
- `qwen-flash` (混合推理模式，需启用)
- `qwen-turbo` (混合推理模式，需启用)
- `qwen3-235b-a22b-thinking-2507` (纯推理模式)
- `qwen3-30b-a3b-thinking-2507` (纯推理模式)
- `qwq-plus` (纯推理模式)

**字段名称**: `reasoning_content`

**API 响应格式**:
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "最终答案",
      "reasoning_content": "思考过程..."
    }
  }]
}
```

**启用方式**:
- 混合推理模式：设置 `enable_thinking: true`
- 纯推理模式：默认启用，无法关闭

**官方文档**: https://www.alibabacloud.com/help/en/model-studio/deep-thinking

**当前实现状态**: ✅ **已支持**
- Aliyun 使用 OpenAI 兼容格式
- 字段名为 `reasoning_content`
- 自动提取到 ChatResponse.reasoning_content

**测试建议**: 使用 qwen-plus 模型并设置 enable_thinking=true

---

### 6. Anthropic (Claude) ❓ 未知

**推理模型**: 
- Claude 3.5 Sonnet (可能有内部推理，但不暴露)

**字段名称**: 未知

**当前实现状态**: ❌ **不支持**
- Anthropic 目前不公开推理过程
- 可能在未来版本中添加

---

### 7. Tencent (腾讯混元) ❓ 未知

**推理模型**: 未知

**当前实现状态**: ❌ **不支持**

---

### 8. Volcengine (火山引擎) ❓ 未知

**推理模型**: 未知

**当前实现状态**: ❌ **不支持**

---

### 9. LongCat ❓ 未知

**推理模型**: 未知

**当前实现状态**: ❌ **不支持**

---

### 10. Ollama ❓ 取决于模型

**推理模型**: 取决于底层模型

**当前实现状态**: ❌ **不支持**
- Ollama 是本地模型服务器
- 支持情况取决于具体模型

---

## 📋 总结表格

| Provider | 推理模型 | 字段名 | 格式 | 当前状态 | 优先级 |
|----------|----------|--------|------|----------|--------|
| **OpenAI** | o1 系列 | reasoning_content | 标准字段 | ✅ 已支持 | - |
| **DeepSeek** | deepseek-reasoner | reasoning_content | 标准字段 | ✅ 已支持 | - |
| **Moonshot** | kimi-thinking-preview | reasoning_content | 标准字段 | ✅ 已支持 | - |
| **Aliyun** | qwen-plus 等 | reasoning_content | 标准字段 | ✅ 已支持 | - |
| **Zhipu** | glm-z1 | 嵌入 content | 标记分隔 | ✅ 已支持 | - |
| Anthropic | - | - | - | ❌ 不支持 | - |
| Tencent | - | - | - | ❌ 不支持 | - |
| Volcengine | - | - | - | ❌ 不支持 | - |
| LongCat | - | - | - | ❌ 不支持 | - |
| Ollama | 取决于模型 | - | - | ❌ 不支持 | - |

---

## 🔧 改进建议

### ✅ 已完成

#### 1. Zhipu GLM-Z1 支持 ✅

**状态**: 已实现

**实现方式**:
```rust
fn extract_zhipu_reasoning_content(content: &str) -> (Option<String>, String) {
    if content.contains("###Thinking") && content.contains("###Response") {
        let parts: Vec<&str> = content.split("###Response").collect();
        if parts.len() >= 2 {
            let thinking = parts[0]
                .replace("###Thinking", "")
                .trim()
                .to_string();
            let response = parts[1..].join("###Response").trim().to_string();

            if !thinking.is_empty() {
                return (Some(thinking), response);
            }
        }
    }
    (None, content.to_string())
}
```

**效果**:
- ✅ 自动分离推理过程和最终答案
- ✅ 统一的 API: `response.reasoning_content` 和 `response.content`
- ✅ 5 个单元测试全部通过

#### 2. Aliyun Qwen Reasoning 支持 ✅

**状态**: 已支持（使用 OpenAI 兼容格式）

**发现**:
- Aliyun 使用 `reasoning_content` 字段（与 OpenAI 相同）
- 混合推理模式需要设置 `enable_thinking: true`
- 纯推理模式（qwq-plus 等）默认启用

**支持的模型**:
- qwen-plus, qwen-flash, qwen-turbo (混合模式)
- qwen3-235b-a22b-thinking-2507 (纯推理)
- qwen3-30b-a3b-thinking-2507 (纯推理)
- qwq-plus (纯推理)

---

### 低优先级

#### 3. 流式响应中的 reasoning_content

**问题**: 流式响应中如何处理推理内容？

**当前状态**:
- OpenAI/DeepSeek/Moonshot: 在 delta 中逐步返回
- Zhipu: 需要缓冲整个 content 后再解析

**改进建议**:
- 为 Zhipu 实现流式解析器
- 检测 `###Thinking` 和 `###Response` 标记
- 动态切换输出目标

---

## 🧪 测试计划

### 1. OpenAI o1 测试
```bash
OPENAI_API_KEY="sk-..." cargo run --example test_openai_reasoning
```

### 2. DeepSeek Reasoner 测试
```bash
DEEPSEEK_API_KEY="sk-..." cargo run --example test_deepseek
```

### 3. Moonshot Thinking 测试
```bash
MOONSHOT_API_KEY="sk-..." cargo run --example test_moonshot_thinking
```

### 4. Zhipu GLM-Z1 测试
```bash
ZHIPU_API_KEY="..." cargo run --example test_zhipu_reasoning
```

---

## 📝 实现清单

- [x] OpenAI reasoning_content 支持
- [x] DeepSeek reasoning_content 支持
- [x] Moonshot reasoning_content 支持（自动支持，因为 OpenAI 兼容）
- [x] Zhipu GLM-Z1 推理内容解析（已实现标记分隔解析）
- [x] Aliyun Qwen 推理内容支持（已支持，使用 OpenAI 兼容格式）
- [ ] 流式推理内容优化（Zhipu 流式解析）
- [ ] 推理内容测试示例
- [ ] 验证所有 providers 的 reasoning_content 提取

---

## 🎯 结论

**当前状态**:
- ✅ **5 个 providers 已支持 reasoning_content**
  - OpenAI (o1 系列)
  - DeepSeek (deepseek-reasoner)
  - Moonshot (kimi-thinking-preview)
  - Aliyun (qwen-plus, qwq-plus 等)
  - Zhipu (glm-z1, GLM-4.1V-Thinking)
- ❌ 5 个 providers 暂不支持（Anthropic, Tencent, Volcengine, LongCat, Ollama）

**已完成的工作**:
1. ✅ 添加 reasoning_content 字段到 ChatResponse
2. ✅ OpenAI protocol 自动提取 reasoning_content
3. ✅ Zhipu GLM-Z1 标记分隔解析
4. ✅ 验证 Aliyun 使用标准 reasoning_content 字段
5. ✅ 单元测试覆盖

**下一步行动**:
1. **中优先级**: 优化 Zhipu 流式推理内容处理
2. **低优先级**: 添加推理内容测试示例
3. **低优先级**: 验证所有 providers 的实际行为

**已实现收益**:
- ✅ 统一的推理内容 API
- ✅ 更好的用户体验
- ✅ 支持 5 个 providers 的推理模型
- ✅ 自动处理不同格式（标准字段 vs 标记分隔）

---

**相关文档**:
- OpenAI o1 文档: https://platform.openai.com/docs/guides/reasoning
- DeepSeek 文档: https://api-docs.deepseek.com/
- Moonshot 文档: https://platform.moonshot.ai/docs/guide/use-kimi-thinking-preview-model
- Zhipu 文档: https://open.bigmodel.cn/dev/api/Reasoning-models/glm-z1

