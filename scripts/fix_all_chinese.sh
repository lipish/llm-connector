#!/bin/bash
# Comprehensive script to fix all remaining Chinese comments in source code

cd /Users/mac-m4/github/llm-connector

echo "Fixing all Chinese comments in source code..."

# Fix all mixed Chinese-English patterns
find src -name "*.rs" -type f -exec sed -i '' \
  -e 's/重用现有类型，保持兼容性/Reuse existing types, maintain compatibility/g' \
  -e 's/它只关注API格式Convert，不涉及具体网络通信。/It only focuses on API format conversion, not specific network communication./g' \
  -e 's/Get聊天完成endpointURL/Get chat completion endpoint URL/g' \
  -e 's/映射HTTPErrorsto统一Errors类型/Map HTTP errors to unified error type/g' \
  -e 's/Getauthentication头 (optional)/Get authentication headers (optional)/g' \
  -e 's/默认Use通用SSE流Parse/Default to use generic SSE stream parser/g' \
  -e 's/服务Provide商trait - Define统一服务接口/Service Provider trait - Define unified service interface/g' \
  -e 's/thistrait代表a具体LLM服务Provide商，Provide完整服务功能。/This trait represents a specific LLM service provider, providing complete service functionality./g' \
  -e 's/它is用户直接交互接口。/It is the direct user interaction interface./g' \
  -e 's/Provide商name (such as "openai", "aliyun", "ollama")/Provider name (such as "openai", "aliyun", "ollama")/g' \
  -e 's/聊天完成/Chat completion/g' \
  -e 's/类型ConvertSupport (for特殊功能访问)/Type conversion support (for special feature access)/g' \
  -e 's/this结构体as大多数标准LLM APIProvide通用实现。/This struct provides generic implementation for most standard LLM APIs./g' \
  -e 's/它UseProtocol traitto处理API特定格式Convert，/It uses Protocol trait to handle API-specific format conversion,/g' \
  -e 's/UseHttpClientto处理网络通信。/uses HttpClient to handle network communication./g' \
  -e 's/Create新通用Provide商/Create new generic provider/g' \
  -e 's/CheckHTTP状态/Check HTTP status/g' \
  -e 's/Provider Build器 - 统一Build接口/Provider Builder - Unified Build Interface/g' \
  -e 's/this模块Providea优雅 Builder 模式 API，forBuild各种 Provider。/This module provides an elegant Builder pattern API for building various Providers./g' \
  -e 's/Provider Build器/Provider Builder/g' \
  -e 's/Create新 Provider Build器/Create new Provider Builder/g' \
  -e 's/基础 URL/Base URL/g' \
  -e 's/Set timeout时间（秒）/Set timeout (seconds)/g' \
  -e 's/60秒超时/60 seconds timeout/g' \
  -e 's/添加Additional HTTP headers/Add additional HTTP headers/g' \
  -e 's/合并authentication头andAdditionalheaders/Merge authentication headers and additional headers/g' \
  -e 's/Create通用Provide商/Create generic provider/g' \
  -e 's/格式: sk-ant-.../Format: sk-ant-.../g' \
  -e 's/Create Zhipu GLM client (OpenAI兼容模式)/Create Zhipu GLM client (OpenAI compatible mode)/g' \
  -e 's/格式: ak_.../Format: ak_.../g' \
  -e 's/VolcengineUse OpenAI 兼容 API 格式，但endpoint路径不同/Volcengine uses OpenAI compatible API format, but with different endpoint paths/g' \
  -e 's/Volcengine API 密钥 (UUID 格式)/Volcengine API key (UUID format)/g' \
  -e 's/Tencent HunyuanUse OpenAI 兼容 API 格式/Tencent Hunyuan uses OpenAI compatible API format/g' \
  -e 's/Tencent Hunyuan API 密钥 (格式: sk-...)/Tencent Hunyuan API key (format: sk-...)/g' \
  -e 's/Create带有customconfigurationTencent Hunyuanclient/Create Tencent Hunyuan client with custom configuration/g' \
  -e 's/Moonshot API 密钥 (格式: sk-...)/Moonshot API key (format: sk-...)/g' \
  -e 's/DeepSeek API 密钥 (格式: sk-...)/DeepSeek API key (format: sk-...)/g' \
  -e 's/if底层ProviderisOllamaProvider，ReturnsSome引用，if则ReturnsNone/If underlying Provider is OllamaProvider, returns Some reference, otherwise returns None/g' \
  -e 's/可以访问 Ollama 特定功能/Can access Ollama-specific features/g' \
  -e 's/尝试将clientConvertasOpenAIProvider/Try to convert client to OpenAIProvider/g' \
  -e 's/尝试将clientConvertasAliyunProvider/Try to convert client to AliyunProvider/g' \
  -e 's/尝试将clientConvertasAnthropicProvider/Try to convert client to AnthropicProvider/g' \
  -e 's/尝试将clientConvertasZhipuProvider/Try to convert client to ZhipuProvider/g' \
  {} \;

echo "Fixed core modules and client"

# Fix Zhipu provider specific patterns
find src/providers -name "*.rs" -type f -exec sed -i '' \
  -e 's/Zhipu GLM服务Provide商实现 - V2架构/Zhipu GLM Service Provider Implementation - V2 Architecture/g' \
  -e 's/this模块ProvideZhipu GLM服务完整实现，Support原生格式andOpenAI兼容格式。/This module provides complete Zhipu GLM service implementation, supporting native format and OpenAI compatible format./g' \
  -e 's/from Zhipu response中提取推理content/Extract reasoning content from Zhipu response/g' \
  -e 's/Zhipu GLM-Z1 etc.推理model将推理过程嵌入in content 中，Use标记分隔：/Zhipu GLM-Z1 and other reasoning models embed reasoning process in content, using markers to separate:/g' \
  -e 's/标记推理过程开始/Marks the start of reasoning process/g' \
  -e 's/标记最终答案开始/Marks the start of final answer/g' \
  -e 's/原始 content 字符串/Original content string/g' \
  -e 's/推理contentand最终答案/Reasoning content and final answer/g' \
  -e 's/CheckisifContains推理标记/Check if contains reasoning markers/g' \
  -e 's/分离推理contentand答案/Separate reasoning content and answer/g' \
  -e 's/if没有推理标记，Returns原始content/If no reasoning markers, return original content/g' \
  -e 's/Zhipu streamingresponse处理阶段/Zhipu streaming response processing stage/g' \
  -e 's/初始状态，etc.待检测isifas推理model/Initial state, waiting to detect if is reasoning model/g' \
  -e 's/in推理阶段（###Thinking 之后，###Response 之前）/In reasoning stage (after ###Thinking, before ###Response)/g' \
  -e 's/in答案阶段（###Response 之后）/In answer stage (after ###Response)/g' \
  -e 's/Zhipu streamingresponse状态机/Zhipu streaming response state machine/g' \
  -e 's/缓冲区，for累积content/Buffer for accumulating content/g' \
  -e 's/当前处理阶段/Current processing stage/g' \
  -e 's/处理streamingcontent增量/Process streaming content delta/g' \
  -e 's/推理content增量and答案content增量/Reasoning content delta and answer content delta/g' \
  -e 's/检测isifContains ###Thinking 标记/Detect if contains ###Thinking marker/g' \
  -e 's/移除标记并进入推理阶段/Remove marker and enter reasoning stage/g' \
  -e 's/Checkisif立即Contains ###Response（完整推理ina块中）/Check if immediately contains ###Response (complete reasoning in one chunk)/g' \
  -e 's/Returns当前缓冲区作as推理content/Return current buffer as reasoning content/g' \
  -e 's/不is推理model，直接Returnscontent/Not a reasoning model, return content directly/g' \
  -e 's/检测isifContains ###Response 标记/Detect if contains ###Response marker/g' \
  -e 's/继续累积推理content/Continue accumulating reasoning content/g' \
  -e 's/in答案阶段，直接Returnscontent/In answer stage, return content directly/g' \
  -e 's/处理 ###Response 标记/Process ###Response marker/g' \
  -e 's/推理部分（###Response 之前）/Reasoning part (before ###Response)/g' \
  -e 's/答案部分（###Response 之后）/Answer part (after ###Response)/g' \
  -e 's/不应该发生，但as安全/Should not happen, but for safety/g' \
  -e 's/Zhipu GLM私有protocol实现/Zhipu GLM private protocol implementation/g' \
  -e 's/智谱SupportOpenAI兼容格式，但有自己authenticationandErrors处理。/Zhipu supports OpenAI compatible format, but has its own authentication and error handling./g' \
  -e 's/由于这is私有protocol，Defineinprovider内部而不is公开protocols模块中。/Since this is a private protocol, it is defined inside the provider rather than in the public protocols module./g' \
  -e 's/Create新智谱Protocol instance (Use原生格式)/Create new Zhipu Protocol instance (using native format)/g' \
  -e 's/CreateUseOpenAI兼容格式智谱Protocol instance/Create Zhipu Protocol instance using OpenAI compatible format/g' \
  -e 's/isifUseOpenAI兼容格式/Whether to use OpenAI compatible format/g' \
  -e 's/Note: Content-Type 由 HttpClient::post()  .json() method自动Set/Note: Content-Type is automatically set by HttpClient::post() .json() method/g' \
  -e 's/不要in这里repetitiveSet，if则may导致repetitiveheadersErrors/Do not set repeatedly here, otherwise may cause duplicate headers error/g' \
  -e 's/智谱UseOpenAI兼容格式/Zhipu uses OpenAI compatible format/g' \
  -e 's/Zhipu Use纯文本格式/Zhipu uses plain text format/g' \
  -e 's/提取推理content（if存in）/Extract reasoning content (if exists)/g' \
  -e 's/智谱专用streamingParse器/Zhipu-specific streaming parser/g' \
  -e 's/智谱 API Use单换行分隔 SSE 事件，而不is标准双换行/Zhipu API uses single newline to separate SSE events, not standard double newline/g' \
  -e 's/格式: data: {...}\\n 而不is data: {...}\\n\\n/Format: data: {...}\\n instead of data: {...}\\n\\n/g' \
  -e 's/智谱Use单换行分隔each data: 行/Zhipu uses single newline to separate each data: line/g' \
  -e 's/跳过空行/Skip empty lines/g' \
  -e 's/提取 data: 后content/Extract content after data:/g' \
  -e 's/跳过 \[DONE\] 标记/Skip \[DONE\] marker/g' \
  -e 's/跳过空 payload/Skip empty payload/g' \
  -e 's/将 JSON 字符串流Convertas StreamingResponse 流/Convert JSON string stream to StreamingResponse stream/g' \
  -e 's/Use状态机处理 Zhipu  ###Thinking and ###Response 标记/Use state machine to handle Zhipu ###Thinking and ###Response markers/g' \
  -e 's/处理推理content标记/Process reasoning content markers/g' \
  -e 's/Use状态机处理content/Use state machine to process content/g' \
  -e 's/更新 delta/Update delta/g' \
  -e 's/同时更新 response.content/Also update response.content/g' \
  {} \;

echo "Fixed Zhipu provider"

echo "Cleanup complete!"
echo "Running tests to verify..."

