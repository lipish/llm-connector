#!/usr/bin/env python3
"""
Translate Chinese comments in Rust files to English
"""

import re
import sys
from pathlib import Path

# Translation dictionary
TRANSLATIONS = {
    # Module level
    "V2统一客户端 - 下一代LLM客户端接口": "V2 Unified Client - Next-generation LLM client interface",
    "这个模块提供统一的客户端接口，支持所有LLM服务提供商": "This module provides a unified client interface supporting all LLM service providers",
    
    # Struct/Type descriptions
    "统一LLM客户端": "Unified LLM Client",
    "这个客户端提供统一的接口来访问各种LLM服务": "This client provides a unified interface to access various LLM services",
    "使用V2架构的清晰抽象层": "using V2 architecture's clean abstraction layer",
    
    # Common section headers
    "示例": "Example",
    "参数": "Parameters",
    "返回": "Returns",
    "错误": "Errors",
    
    # Provider names
    "阿里云DashScope": "Aliyun DashScope",
    "智谱GLM": "Zhipu GLM",
    "火山引擎": "Volcengine",
    "腾讯云混元": "Tencent Hunyuan",
    "月之暗面": "Moonshot",
    
    # Common phrases - Create
    "从任何Provider创建客户端": "Create client from any Provider",
    "创建OpenAI客户端": "Create OpenAI client",
    "创建带有自定义基础URL的OpenAI客户端": "Create OpenAI client with custom base URL",
    "创建Azure OpenAI客户端": "Create Azure OpenAI client",
    "创建阿里云DashScope客户端": "Create Aliyun DashScope client",
    "创建Anthropic Claude客户端": "Create Anthropic Claude client",
    "创建智谱GLM客户端": "Create Zhipu GLM client",
    "创建智谱GLM客户端 (OpenAI兼容模式)": "Create Zhipu GLM client (OpenAI compatible mode)",
    "创建Ollama客户端 (默认本地地址)": "Create Ollama client (default local address)",
    "创建带有自定义URL的Ollama客户端": "Create Ollama client with custom URL",
    "创建OpenAI兼容服务客户端": "Create OpenAI-compatible service client",
    "创建LongCat Anthropic格式客户端": "Create LongCat Anthropic format client",
    "创建带有自定义配置的LongCat Anthropic客户端": "Create LongCat Anthropic client with custom configuration",
    "创建火山引擎（Volcengine）客户端": "Create Volcengine client",
    "创建带有自定义配置的火山引擎客户端": "Create Volcengine client with custom configuration",
    "创建腾讯云混元（Tencent Hunyuan）客户端": "Create Tencent Hunyuan client",
    "创建带有自定义配置的腾讯云混元客户端": "Create Tencent Hunyuan client with custom configuration",
    "创建 Moonshot（月之暗面）客户端": "Create Moonshot client",
    "创建带有自定义配置的 Moonshot 客户端": "Create Moonshot client with custom configuration",
    "创建 DeepSeek 客户端": "Create DeepSeek client",
    "创建带有自定义配置的 DeepSeek 客户端": "Create DeepSeek client with custom configuration",
    
    # Advanced constructors
    "高级构造函数 - 自定义配置": "Advanced Constructors - Custom Configuration",
    "创建带有自定义配置的OpenAI客户端": "Create OpenAI client with custom configuration",
    "创建带有自定义配置的Aliyun客户端": "Create Aliyun client with custom configuration",
    "创建Aliyun国际版客户端": "Create Aliyun international client",
    "创建Aliyun专有云客户端": "Create Aliyun private cloud client",
    "创建带有自定义超时的Aliyun客户端": "Create Aliyun client with custom timeout",
    "创建带有自定义配置的Anthropic客户端": "Create Anthropic client with custom configuration",
    "创建Anthropic Vertex AI客户端": "Create Anthropic Vertex AI client",
    "创建Anthropic AWS Bedrock客户端": "Create Anthropic AWS Bedrock client",
    "创建带有自定义超时的Anthropic客户端": "Create Anthropic client with custom timeout",
    "创建带有自定义配置的Zhipu客户端": "Create Zhipu client with custom configuration",
    "创建带有自定义超时的Zhipu客户端": "Create Zhipu client with custom timeout",
    "创建Zhipu企业版客户端": "Create Zhipu enterprise client",
    "创建带有自定义配置的Ollama客户端": "Create Ollama client with custom configuration",
    
    # Methods
    "获取提供商名称": "Get provider name",
    "发送聊天完成请求": "Send chat completion request",
    "发送流式聊天完成请求": "Send streaming chat completion request",
    "获取可用模型列表": "Get available models list",
    "获取底层提供商的引用 (用于特殊功能访问)": "Get reference to underlying provider (for special feature access)",
    "类型安全的Provider转换方法": "Type-safe Provider conversion methods",
    "尝试将客户端转换为OllamaProvider": "Try to convert client to OllamaProvider",
    
    # Parameter descriptions
    "OpenAI API密钥": "OpenAI API key",
    "API密钥": "API key",
    "自定义基础URL": "Custom base URL",
    "Azure OpenAI API密钥": "Azure OpenAI API key",
    "Azure OpenAI端点": "Azure OpenAI endpoint",
    "API版本": "API version",
    "Anthropic API密钥 (格式: sk-ant-...)": "Anthropic API key (format: sk-ant-...)",
    "服务基础URL": "Service base URL",
    "服务名称": "Service name",
    "Ollama服务的URL": "Ollama service URL",
    "聊天请求": "Chat request",
    "聊天响应": "Chat response",
    "聊天流": "Chat stream",
    "模型名称列表": "List of model names",
    
    # Return descriptions
    "如果底层Provider是OllamaProvider，返回Some引用，否则返回None": "Returns Some reference if underlying Provider is OllamaProvider, otherwise None",
    "可以进行类型转换以访问特定提供商的功能": "Can perform type conversion to access provider-specific features",
    
    # Common actions
    "创建请求": "Create request",
    "发送请求": "Send request",
    "创建": "Create",
    "获取": "Get",
    "设置": "Set",
    "检查": "Check",
    "验证": "Validate",
    "转换": "Convert",
    "解析": "Parse",
    "构建": "Build",
    "初始化": "Initialize",
    
    # Descriptions
    "火山引擎使用 OpenAI 兼容的 API 格式，但端点路径不同": "Volcengine uses OpenAI-compatible API format, but with different endpoint paths",
    "腾讯云混元使用 OpenAI 兼容的 API 格式": "Tencent Hunyuan uses OpenAI-compatible API format",
    "Moonshot 使用 OpenAI 兼容的 API 格式": "Moonshot uses OpenAI-compatible API format",
    "DeepSeek 使用 OpenAI 兼容的 API 格式，支持推理模型": "DeepSeek uses OpenAI-compatible API format, supports reasoning models",
    "LongCat的Anthropic端点使用Bearer认证而不是标准的x-api-key认证": "LongCat's Anthropic endpoint uses Bearer authentication instead of standard x-api-key authentication",

    # Additional translations
    "支持多模态内容，包括文本、图片等": "Supports multi-modal content including text, images, etc.",
    "一条消息可以包含多个内容块，支持文本、图片等多模态内容": "A message can contain multiple content blocks, supporting multi-modal content like text, images, etc.",
    "可配置的协议适配器 - 配置驱动的抽象": "Configurable Protocol Adapter - Configuration-driven abstraction",
    "这个模块提供了一个通用的协议适配器，通过配置来定制行为，": "This module provides a generic protocol adapter that customizes behavior through configuration,",
    "可配置的协议适配器": "Configurable Protocol Adapter",
    "包装一个基础协议，通过配置来修改其行为（端点路径、认证方式等）。": "Wraps a base protocol and modifies its behavior through configuration (endpoint paths, authentication methods, etc.).",
    "协议配置": "Protocol Configuration",
    "定义协议的静态配置，包括名称、端点、认证方式等。": "Defines static configuration for the protocol, including name, endpoints, authentication methods, etc.",
    "端点配置": "Endpoint Configuration",
    "定义 API 端点的路径模板，支持 `{base_url}` 变量替换。": "Defines API endpoint path templates, supporting `{base_url}` variable substitution.",
    "支持变量: `{base_url}`": "Supports variable: `{base_url}`",
    "认证配置": "Authentication Configuration",
    "定义如何处理 API 认证。": "Defines how to handle API authentication.",
    "Create新的可配置协议适配器": "Create new configurable protocol adapter",
    "Create一个使用标准 OpenAI 端点和 Bearer 认证的配置。": "Create a configuration using standard OpenAI endpoints and Bearer authentication.",
    "提供统一的HTTP通信层，支持标准请求和流式请求。": "Provides unified HTTP communication layer, supporting standard and streaming requests.",
    "封装了HTTP通信的所有细节，包括认证、超时、代理等配置。": "Encapsulates all HTTP communication details, including authentication, timeout, proxy configuration, etc.",

    # More common phrases
    "支持": "Support",
    "包含": "Contains",
    "定义": "Define",
    "提供": "Provide",
    "使用": "Use",
    "通过": "Through",
    "包括": "Including",
    "等": "etc.",

    # Additional core terms
    "协议名称": "Protocol name",
    "模型列表端点模板（可选）": "Model list endpoint template (optional)",
    "基础协议实例": "Base protocol instance",
    "便捷构造器 - OpenAI 兼容协议": "Convenience constructor - OpenAI compatible protocol",
    "从内部协议提取 token": "Extract token from internal protocol",
    "这是一个辅助方法，用于从内部协议的认证头中提取 token。": "This is a helper method to extract token from the internal protocol's authentication headers.",
    "HTTP客户端实现 - V2架构": "HTTP Client Implementation - V2 Architecture",
    "HTTP客户端": "HTTP Client",
    "Create新的HTTP客户端": "Create new HTTP client",
    "Create带有自Define配置的HTTP客户端": "Create HTTP client with custom configuration",
    "添加请求头": "Add request headers",
    "添加单个请求头": "Add single request header",
    "发送GET请求": "Send GET request",
    "添加所有配置的请求头": "Add all configured request headers",
    "发送POST请求": "Send POST request",
    "发送流式POST请求": "Send streaming POST request",
    "发送带有自Define头的POST请求": "Send POST request with custom headers",
    "再添加配置的请求头 (可能会覆盖自Define头)": "Then add configured headers (may override custom headers)",
    "HTTP客户端实现": "HTTP client implementation",
    "协议trait - Define纯API规范": "Protocol trait - Defines pure API specification",
    "这个trait代表一个LLM API的协议规范，如OpenAI API、Anthropic APIetc.。": "This trait represents an LLM API protocol specification, such as OpenAI API, Anthropic API, etc.",
    "协议特定的请求类型": "Protocol-specific request type",
    "协议特定的响应类型": "Protocol-specific response type",
    "协议名称 (如 \"openai\", \"anthropic\")": "Protocol name (e.g., \"openai\", \"anthropic\")",
    "Get模型列表的端点URL (可选)": "Get model list endpoint URL (optional)",
    "Build协议特定的请求": "Build protocol-specific request",
    "Parse协议特定的响应": "Parse protocol-specific response",
    "Parse模型列表响应": "Parse model list response",
    "Parse流式响应 (可选)": "Parse streaming response (optional)",
    "流式聊天完成": "Streaming chat completion",
    "Get协议引用": "Get protocol reference",
    "Get客户端引用": "Get client reference",
    "Parse响应": "Parse response",
    "Provide链式调用的 API 来Build Provider，统一处理所有配置项。": "Provides fluent API to build Provider, handling all configuration items uniformly.",
    "协议实例": "Protocol instance",
    "注意：这些头部会与协议的认证头部合并。": "Note: These headers will be merged with the protocol's authentication headers.",
    "配置好的 GenericProvider 实例": "Configured GenericProvider instance",
    "如果 HTTP 客户端Create失败，ReturnsErrors": "Returns error if HTTP client creation fails",
    "Create HTTP 客户端": "Create HTTP client",
    "CreateAliyun DashScope客户端": "Create Aliyun DashScope client",
    "CreateZhipu GLM客户端": "Create Zhipu GLM client",
    "CreateZhipu GLM客户端 (OpenAI兼容模式)": "Create Zhipu GLM client (OpenAI compatible mode)",
    "CreateVolcengine（Volcengine）客户端": "Create Volcengine client",
    "Create带有自Define配置的Volcengine客户端": "Create Volcengine client with custom configuration",
    "CreateTencent Hunyuan（Tencent Hunyuan）客户端": "Create Tencent Hunyuan client",

    # More patterns
    "模型": "model",
    "配置": "configuration",
    "客户端": "client",
    "协议": "protocol",
    "适配器": "adapter",
    "提供商": "provider",
    "请求": "request",
    "响应": "response",
    "流式": "streaming",
    "工具": "tool",
    "函数": "function",
    "调用": "call",
    "端点": "endpoint",
    "认证": "authentication",
    "头部": "headers",
    "实例": "instance",
    "方法": "method",
    "辅助": "helper",
    "构造器": "constructor",
    "便捷": "convenience",
    "自Define": "custom",
    "自定义": "custom",
    "可选": "optional",
    "注意": "Note",
    "如果": "if",
    "失败": "fails",
    "成功": "success",
    "错误": "error",
    "返回": "return",
    "等。": "etc.",
    "如": "such as",
    "例如": "e.g.",
    "或": "or",
    "和": "and",
    "的": "",
    "了": "",
    "与": "with",
    "为": "as",
    "是": "is",
    "在": "in",
    "从": "from",
    "到": "to",
    "会": "will",
    "可能": "may",
    "所有": "all",
    "这个": "this",
    "这些": "these",
    "一个": "a",
    "用于": "for",
    "来": "to",

    # More specific translations
    "消息内容块Define": "Message Content Block Definition",
    "消息内容块": "Message Content Block",
    "文本块": "Text block",
    "图片块（Base64）": "Image block (Base64)",
    "图片块（URL）": "Image block (URL)",
    "图片块（Anthropic 格式）": "Image block (Anthropic format)",
    "图片 URL 块（OpenAI 格式）": "Image URL block (OpenAI format)",
    "Create文本块": "Create text block",
    "Create Base64 图片块（Anthropic 格式）": "Create Base64 image block (Anthropic format)",
    "媒体类型，such as \"image/jpeg\", \"image/png\"": "Media type, such as \"image/jpeg\", \"image/png\"",
    "Base64 编码图片数据": "Base64 encoded image data",
    "Create图片 URL 块（Anthropic 格式）": "Create image URL block (Anthropic format)",
    "Create图片 URL 块（OpenAI 格式）": "Create image URL block (OpenAI format)",
    "Create图片 URL 块（OpenAI 格式，带 detail Parameters）": "Create image URL block (OpenAI format, with detail parameter)",
    "图片 URL": "Image URL",
    "图片细节级别，optional值: \"auto\", \"low\", \"high\"": "Image detail level, optional values: \"auto\", \"low\", \"high\"",
    "Get文本内容（ifis文本块）": "Get text content (if is text block)",
    "判断is否as文本块": "Check if is text block",
    "判断is否as图片块": "Check if is image block",
    "图片to源（Anthropic 格式）": "Image source (Anthropic format)",
    "Base64 编码图片": "Base64 encoded image",
    "媒体类型，such as \"image/jpeg\", \"image/png\"": "Media type, such as \"image/jpeg\", \"image/png\"",
    "图片 URL": "Image URL",
    "图片细节级别": "Image detail level",
    "optional值: \"auto\", \"low\", \"high\"": "Optional values: \"auto\", \"low\", \"high\"",
    "Provider-specific reasoning content (GLM 风格)": "Provider-specific reasoning content (GLM style)",
    "Provider-specific reasoning (Qwen/DeepSeek/OpenAI o1 通用键)": "Provider-specific reasoning (Qwen/DeepSeek/OpenAI o1 common key)",
    "Provider-specific thought (OpenAI o1 键)": "Provider-specific thought (OpenAI o1 key)",
    "Provider-specific thinking (Anthropic 键)": "Provider-specific thinking (Anthropic key)",
    "Reasoning (Qwen/DeepSeek/OpenAI o1 通用键)": "Reasoning (Qwen/DeepSeek/OpenAI o1 common key)",
    "Thought (OpenAI o1 键)": "Thought (OpenAI o1 key)",
    "Thinking (Anthropic 键)": "Thinking (Anthropic key)",
    "避免as每个 Provider 编写重复样板代码。": "Avoids writing repetitive boilerplate code for each Provider.",
    "额外静态headers": "Additional static headers",
    "聊天endpoint模板": "Chat endpoint template",
    "例such as: `\"{base_url}/v1/chat/completions\"`": "Example: `\"{base_url}/v1/chat/completions\"`",
    "例such as: `\"{base_url}/v1/models\"`": "Example: `\"{base_url}/v1/models\"`",
    "生成: `Authorization: Bearer {token}`": "Generates: `Authorization: Bearer {token}`",
    "生成: `{header_name}: {token}`": "Generates: `{header_name}: {token}`",
    "Header 名称": "Header name",
    "无authentication": "No authentication",
    "customauthentication（Through闭包）": "Custom authentication (through closure)",
    "闭包接收 token，Returnsheaders列表": "Closure receives token, returns headers list",

    # Common word replacements
    "我is豆包": "I am Doubao",
    "风格": "style",
    "通用键": "common key",
    "键": "key",
    "as": "as",
    "is": "is",
    "if": "if",
    "to": "to",
    "such as": "such as",
    "例": "Example",
    "such as": "such as",
    "编码": "encoded",
    "数据": "data",
    "内容": "content",
    "判断": "Check",
    "否": "if",
    "源": "source",
    "级别": "level",
    "值": "values",
    "额外": "Additional",
    "静态": "static",
    "模板": "template",
    "生成": "Generates",
    "名称": "name",
    "无": "No",
    "接收": "receives",
    "列表": "list",
    "避免": "Avoids",
    "每个": "each",
    "编写": "writing",
    "重复": "repetitive",
    "样板": "boilerplate",
    "代码": "code",
}

def translate_line(line):
    """Translate a single line"""
    for chinese, english in TRANSLATIONS.items():
        if chinese in line:
            line = line.replace(chinese, english)
    return line

def translate_file(filepath):
    """Translate a single file"""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    lines = content.split('\n')
    translated_lines = [translate_line(line) for line in lines]
    translated_content = '\n'.join(translated_lines)
    
    if translated_content != content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(translated_content)
        return True
    return False

def main():
    src_dir = Path('src')
    rust_files = list(src_dir.rglob('*.rs'))
    
    translated_count = 0
    for filepath in rust_files:
        if translate_file(filepath):
            print(f"Translated: {filepath}")
            translated_count += 1
    
    print(f"\nTotal files translated: {translated_count}/{len(rust_files)}")

if __name__ == '__main__':
    main()

