#!/bin/bash

# Script to translate Chinese comments to English in Rust files

# Common translations
declare -A translations=(
    # Module/file level
    ["V2统一客户端 - 下一代LLM客户端接口"]="V2 Unified Client - Next-generation LLM client interface"
    ["这个模块提供统一的客户端接口，支持所有LLM服务提供商"]="This module provides a unified client interface supporting all LLM service providers"
    ["统一LLM客户端"]="Unified LLM Client"
    ["这个客户端提供统一的接口来访问各种LLM服务"]="This client provides a unified interface to access various LLM services"
    ["使用V2架构的清晰抽象层"]="using V2 architecture's clean abstraction layer"
    
    # Common phrases
    ["创建"]="Create"
    ["配置"]="Configuration"
    ["提供"]="Provide"
    ["支持"]="Support"
    ["返回"]="Return"
    ["错误"]="Error"
    ["处理"]="Handle"
    ["请求"]="Request"
    ["响应"]="Response"
    ["参数"]="Parameters"
    ["示例"]="Example"
    ["获取"]="Get"
    ["设置"]="Set"
    ["检查"]="Check"
    ["验证"]="Validate"
    ["转换"]="Convert"
    ["解析"]="Parse"
    ["构建"]="Build"
    ["初始化"]="Initialize"
    
    # Specific providers
    ["阿里云DashScope"]="Aliyun DashScope"
    ["智谱GLM"]="Zhipu GLM"
    ["火山引擎"]="Volcengine"
    ["腾讯云混元"]="Tencent Hunyuan"
    ["月之暗面"]="Moonshot"
    
    # Common patterns
    ["从任何Provider创建客户端"]="Create client from any Provider"
    ["创建OpenAI客户端"]="Create OpenAI client"
    ["创建带有自定义基础URL的OpenAI客户端"]="Create OpenAI client with custom base URL"
    ["创建Azure OpenAI客户端"]="Create Azure OpenAI client"
    ["创建阿里云DashScope客户端"]="Create Aliyun DashScope client"
    ["创建Anthropic Claude客户端"]="Create Anthropic Claude client"
    ["创建智谱GLM客户端"]="Create Zhipu GLM client"
    ["创建Ollama客户端"]="Create Ollama client"
    ["创建带有自定义URL的Ollama客户端"]="Create Ollama client with custom URL"
    ["创建OpenAI兼容服务客户端"]="Create OpenAI-compatible service client"
    ["创建火山引擎（Volcengine）客户端"]="Create Volcengine client"
    ["创建腾讯云混元（Tencent Hunyuan）客户端"]="Create Tencent Hunyuan client"
    ["创建 Moonshot（月之暗面）客户端"]="Create Moonshot client"
    ["创建 DeepSeek 客户端"]="Create DeepSeek client"
    
    # Advanced constructors
    ["高级构造函数 - 自定义配置"]="Advanced Constructors - Custom Configuration"
    ["创建带有自定义配置的"]="Create with custom configuration for"
    ["创建带有自定义超时的"]="Create with custom timeout for"
    
    # Methods
    ["获取提供商名称"]="Get provider name"
    ["发送聊天完成请求"]="Send chat completion request"
    ["发送流式聊天完成请求"]="Send streaming chat completion request"
    ["获取可用模型列表"]="Get available models list"
    ["获取底层提供商的引用"]="Get reference to underlying provider"
    ["类型安全的Provider转换方法"]="Type-safe Provider conversion methods"
    ["尝试将客户端转换为OllamaProvider"]="Try to convert client to OllamaProvider"
    
    # Parameter descriptions
    ["API密钥"]="API key"
    ["自定义基础URL"]="Custom base URL"
    ["Azure OpenAI端点"]="Azure OpenAI endpoint"
    ["API版本"]="API version"
    ["服务基础URL"]="Service base URL"
    ["服务名称"]="Service name"
    ["Ollama服务的URL"]="Ollama service URL"
    ["聊天请求"]="Chat request"
    ["聊天响应"]="Chat response"
    ["聊天流"]="Chat stream"
    ["模型名称列表"]="List of model names"
    
    # Return descriptions
    ["如果底层Provider是OllamaProvider，返回Some引用，否则返回None"]="Returns Some reference if underlying Provider is OllamaProvider, otherwise None"
    ["可以进行类型转换以访问特定提供商的功能"]="Can perform type conversion to access provider-specific features"
)

echo "This script would translate Chinese comments to English"
echo "Due to the large number of files and complex patterns,"
echo "manual translation is recommended for accuracy."
echo ""
echo "Files with Chinese comments:"
find src -name "*.rs" -type f | while read file; do
    if grep -q "创建\|配置\|提供\|支持\|返回\|错误\|处理\|请求\|响应" "$file" 2>/dev/null; then
        echo "  - $file"
    fi
done

