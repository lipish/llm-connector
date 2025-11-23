#!/bin/bash
# Final cleanup of all remaining Chinese characters

cd /Users/mac-m4/github/llm-connector

echo "Final Chinese cleanup..."

# Fix all remaining mixed patterns
find src -name "*.rs" -type f -exec sed -i '' \
  -e 's/OpenAIcompatible模式/OpenAI compatible mode/g' \
  -e 's/Createconfiguration驱动protocol/Create configuration-driven protocol/g' \
  -e 's/Use Bearer authentication + Additionalheaders/Use Bearer authentication + Additional headers/g' \
  -e 's/Use Bearer 而不is x-api-key/Use Bearer instead of x-api-key/g' \
  -e 's/应该Use Bearer authentication/Should use Bearer authentication/g' \
  -e 's/应该Contains anthropic-version/Should contain anthropic-version/g' \
  -e 's/不应该Contains x-api-key/Should not contain x-api-key/g' \
  -e 's/Use ConfigurableProtocol wrap OpenAI protocol，customendpoint路径/Use ConfigurableProtocol to wrap OpenAI protocol, custom endpoint paths/g' \
  -e 's/customendpoint路径/custom endpoint paths/g' \
  -e 's/完全compatiblestandard OpenAI protocol/fully compatible with standard OpenAI protocol/g' \
  -e 's/Ollamaisa本地LLMservice/Ollama is a local LLM service/g' \
  -e 's/具有特殊model管理功能/with special model management features/g' \
  -e 's/因此需要customProviderimplementation/therefore requires custom Provider implementation/g' \
  -e 's/由于Ollama具有特殊model管理功能/Since Ollama has special model management features/g' \
  -e 's/我们UsecustomProviderimplementation/we use custom Provider implementation/g' \
  -e 's/而不isGenericProvider模式/instead of GenericProvider pattern/g' \
  -e 's/Create新OllamaProvider/Create new OllamaProvider/g' \
  -e 's/Content-Type 由 HttpClient::post()  .json() method自动Set/Content-Type is automatically set by HttpClient::post() .json() method/g' \
  -e 's/拉取model/Pull model/g' \
  -e 's/要拉取modelname/Model name to pull/g' \
  -e 's/such as "llama2", "codellama"/such as "llama2", "codellama"/g' \
  -e 's/删除model/Delete model/g' \
  -e 's/要删除modelname/Model name to delete/g' \
  -e 's/Getmodel信息/Get model information/g' \
  -e 's/model详细信息/Model details/g' \
  -e 's/Checkmodelisif存in/Check if model exists/g' \
  -e 's/Ollama不Supporttool角色/Ollama does not support tool role/g' \
  -e 's/Ollama Use纯文本format/Ollama uses plain text format/g' \
  -e 's/Ollama不ReturnstokenUse信息/Ollama does not return token usage information/g' \
  -e 's/OllamaUseJSONLformat而不isSSE/Ollama uses JSONL format instead of SSE/g' \
  -e 's/CreateOllamaserviceProvider (default本地地址)/Create Ollama service Provider (default local address)/g' \
  -e 's/2分钟timeout/2 minutes timeout/g' \
  -e 's/V2serviceProvider模chunk/V2 Service Provider Module/g' \
  -e 's/this模chunkContainsallserviceProviderimplementation/This module contains all service Provider implementations/g' \
  -e 's/eachProvider代表a具体LLMservice/each Provider represents a specific LLM service/g' \
  -e 's/重新导出serviceProvidertypeandfunction/Re-export service Provider types and functions/g' \
  -e 's/DeepSeek Use OpenAI compatible API format/DeepSeek uses OpenAI compatible API format/g' \
  -e 's/Supportreasoningmodel（reasoning content）andstandard对话model/Supports reasoning models (reasoning content) and standard chat models/g' \
  -e 's/Aliyun DashScopeserviceProviderimplementation - V2架构/Aliyun DashScope Service Provider Implementation - V2 Architecture/g' \
  -e 's/this模chunkProvideAliyun DashScopeservicecompleteimplementation/This module provides complete Aliyun DashScope service implementation/g' \
  -e 's/Use统一V2架构/using unified V2 architecture/g' \
  -e 's/Aliyun DashScope私有protocolimplementation/Aliyun DashScope private protocol implementation/g' \
  -e 's/这is阿里云specificAPIformat/This is Aliyun-specific API format/g' \
  -e 's/withOpenAIandAnthropic都不同/different from both OpenAI and Anthropic/g' \
  -e 's/Create新阿里云Protocol instance/Create new Aliyun Protocol instance/g' \
  -e 's/不要in这里repetitiveSet/Do not set repeatedly here/g' \
  -e 's/if则will导致 "Content-Type application\/json,application\/json is not supported" Errors/otherwise will cause "Content-Type application\/json,application\/json is not supported" error/g' \
  -e 's/Convertas阿里云format/Convert to Aliyun format/g' \
  -e 's/Aliyun Use纯文本format/Aliyun uses plain text format/g' \
  -e 's/streaming模式需要 incremental_output/Streaming mode requires incremental_output/g' \
  -e 's/直接Use用户指定values/Directly use user-specified values/g' \
  -e 's/clear已处理行/Clear processed lines/g' \
  -e 's/Build choices 数组（符合 OpenAI standardformat）/Build choices array (conforming to OpenAI standard format)/g' \
  -e 's/from choices\[0\] 提取 content 作as便利字段/Extract content from choices[0] as convenience field/g' \
  -e 's/提取 usage 信息/Extract usage information/g' \
  -e 's/Aliyun 不Provide created 时间戳/Aliyun does not provide created timestamp/g' \
  -e 's/阿里云specificdata结构/Aliyun-specific data structures/g' \
  -e 's/需要特殊处理streamingrequest/Requires special handling for streaming requests/g' \
  -e 's/因as Aliyun 需要 X-DashScope-SSE headers/because Aliyun requires X-DashScope-SSE headers/g' \
  -e 's/GetProtocol instance引用/Get Protocol instance reference/g' \
  -e 's/Get HTTP client引用/Get HTTP client reference/g' \
  -e 's/Create临时client/Create temporary client/g' \
  -e 's/addstreamingheaders/add streaming headers/g' \
  -e 's/configuration好阿里云serviceProviderinstance/Configured Aliyun service Provider instance/g' \
  -e 's/Createwithcustomconfiguration阿里云serviceProvider/Create Aliyun service Provider with custom configuration/g' \
  -e 's/Custom base URL (optional，defaultas官方endpoint)/Custom base URL (optional, defaults to official endpoint)/g' \
  -e 's/CreateHTTP Client（不Containsstreamingheaders）/Create HTTP Client (without streaming headers)/g' \
  -e 's/Createcustom Aliyun Provider（需要特殊处理streamingrequest）/Create custom Aliyun Provider (requires special streaming request handling)/g' \
  -e 's/Createfor阿里云国际版serviceProvider/Create Aliyun international service Provider/g' \
  -e 's/阿里云国际版API key/Aliyun international API key/g' \
  -e 's/区域 (such as "us-west-1", "ap-southeast-1")/Region (such as "us-west-1", "ap-southeast-1")/g' \
  -e 's/Createfor阿里云专有云serviceProvider/Create Aliyun private cloud service Provider/g' \
  -e 's/专有云endpointURL/Private cloud endpoint URL/g' \
  -e 's/Createwithcustomtimeout阿里云serviceProvider/Create Aliyun service Provider with custom timeout/g' \
  -e 's/阿里云某些modelmay需要较长处理时间/Some Aliyun models may require longer processing time/g' \
  -e 's/thisfunctionProvide便利timeoutconfiguration/this function provides convenient timeout configuration/g' \
  -e 's/Set 120 seconds timeout，适for长文本处理/Set 120 seconds timeout, suitable for long text processing/g' \
  -e 's/test显式启用/Test explicit enable/g' \
  -e 's/显式启用/Explicitly enable/g' \
  -e 's/test显式禁用/Test explicit disable/g' \
  -e 's/显式禁用/Explicitly disable/g' \
  -e 's/test未指定（default不启用）/Test unspecified (default not enabled)/g' \
  -e 's/enable_thinking 未指定/enable_thinking not specified/g' \
  -e 's/Tencent Hunyuan uses OpenAI compatible API format/Tencent Hunyuan uses OpenAI compatible API format/g' \
  -e 's/Anthropic ClaudeserviceProviderimplementation - V2架构/Anthropic Claude Service Provider Implementation - V2 Architecture/g' \
  -e 's/this模chunkProvideAnthropic Claudeservicecompleteimplementation/This module provides complete Anthropic Claude service implementation/g' \
  -e 's/configuration好AnthropicserviceProviderinstance/Configured Anthropic service Provider instance/g' \
  -e 's/Custom base URL (optional，defaultas官方endpoint)/Custom base URL (optional, defaults to official endpoint)/g' \
  -e 's/Google Cloud项目ID/Google Cloud project ID/g' \
  -e 's/区域 (such as "us-central1")/Region (such as "us-central1")/g' \
  -e 's/Google Cloud访问令牌/Google Cloud access token/g' \
  -e 's/Vertex AI不需要Anthropic API key/Vertex AI does not require Anthropic API key/g' \
  -e 's/AWS区域 (such as "us-east-1")/AWS region (such as "us-east-1")/g' \
  -e 's/AWS访问keyID/AWS access key ID/g' \
  -e 's/AWS秘密访问key/AWS secret access key/g' \
  -e 's/Bedrock不需要Anthropic API key/Bedrock does not require Anthropic API key/g' \
  -e 's/Note: 这里简化AWS签名process/Note: This simplifies AWS signature process/g' \
  -e 's/实际Use中需要implementationAWS SigV4签名/actual use requires AWS SigV4 signature implementation/g' \
  -e 's/Anthropic某些modelmay需要较长处理时间/Some Anthropic models may require longer processing time/g' \
  -e 's/Set 120 seconds timeout，适for长文本处理/Set 120 seconds timeout, suitable for long text processing/g' \
  -e 's/ifformatcorrectReturnstrue，if则Returnsfalse/Returns true if format is correct, otherwise returns false/g' \
  -e 's/OpenAIserviceProviderimplementation - V2架构/OpenAI Service Provider Implementation - V2 Architecture/g' \
  -e 's/this模chunkProvideOpenAIservicecompleteimplementation/This module provides complete OpenAI service implementation/g' \
  -e 's/configuration好OpenAIserviceProviderinstance/Configured OpenAI service Provider instance/g' \
  -e 's/thisfunctionas各种OpenAIcompatibleserviceProvide便利Createmethod/This function provides convenient create method for various OpenAI-compatible services/g' \
  -e 's/Service name (forErrors消息)/Service name (for error messages)/g' \
  {} \;

echo "Final cleanup complete!"

