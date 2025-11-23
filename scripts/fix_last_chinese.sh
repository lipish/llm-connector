#\!/bin/bash
cd /Users/mac-m4/github/llm-connector

find src -name "*.rs" -type f -exec sed -i '' \
  -e 's/由于Ollamawith special model management features，we use custom Provider implementation/Since Ollama has special model management features, we use custom Provider implementation/g' \
  -e 's/要Pull modelname (such as "llama2", "codellama")/Model name to pull (such as "llama2", "codellama")/g' \
  -e 's/要Delete modelname/Model name to delete/g' \
  -e 's/导出standardprotocol/Export standard protocols/g' \
  -e 's/导出私有protocol（from providers 中）/Export private protocols (from providers)/g' \
  -e 's/protocol模chunk - 公开standardprotocol/Protocol Module - Public Standard Protocols/g' \
  -e 's/this模chunk只Contains业界公认standardLLM APIprotocol：/This module only contains industry-recognized standard LLM API protocols:/g' \
  -e 's/- \*\*OpenAI Protocol\*\*: standardOpenAI API规范 - 被多\/serviceProviderSupport/- **OpenAI Protocol**: Standard OpenAI API specification - supported by multiple service providers/g' \
  -e 's/- \*\*Anthropic Protocol\*\*: standardAnthropic Claude API规范 - 官方protocol/- **Anthropic Protocol**: Standard Anthropic Claude API specification - official protocol/g' \
  -e 's/## 设计原则/## Design Principles/g' \
  -e 's/- 只Contains公开、standard化protocol/- Only contains public, standardized protocols/g' \
  -e 's/- 其他serviceProvidermaywillimplementationtheseprotocol/- Other service providers may implement these protocols/g' \
  -e 's/- 私有protocolDefinein各自 `providers` 模chunk中/- Private protocols are defined in respective `providers` modules/g' \
  -e 's/Note：具体serviceProviderimplementationin `providers` 模chunk中。/Note: Specific service provider implementations are in the `providers` module./g' \
  -e 's/重新导出standardprotocoltype/Re-export standard protocol types/g' \
  -e 's/Anthropic Claudeprotocolimplementation - V2架构/Anthropic Claude Protocol Implementation - V2 Architecture/g' \
  -e 's/this模chunkimplementationAnthropic Claude APIprotocol规范。/This module implements the Anthropic Claude API protocol specification./g' \
  -e 's/Create新AnthropicProtocol instance/Create new Anthropic Protocol instance/g' \
  -e 's/Anthropic API 需要分离 system 消息/Anthropic API requires separating system messages/g' \
  -e 's/Anthropic 只Supporta system 消息，放in单独字段中/Anthropic only supports one system message, placed in a separate field/g' \
  -e 's/if有多\/ system 消息，合并它们/If there are multiple system messages, merge them/g' \
  -e 's/Anthropic 总isUse数组format/Anthropic always uses array format/g' \
  -e 's/Anthropic 暂不Support tool 角色，Convertas user/Anthropic does not support tool role yet, convert to user/g' \
  -e 's/Anthropic 要求必须Set/Anthropic requires this to be set/g' \
  -e 's/Anthropic Returns单\/contentchunk/Anthropic returns single content chunk/g' \
  -e 's/Anthropic Use不同streamingformat：/Anthropic uses different streaming format:/g' \
  -e 's/- message_start: Contains message 对象（有 id）/- message_start: Contains message object (with id)/g' \
  -e 's/- content_block_start: 开始contentchunk/- content_block_start: Start content chunk/g' \
  -e 's/- content_block_stop: 结束contentchunk/- content_block_stop: End content chunk/g' \
  -e 's/- message_delta: 消息delta（Contains usage）/- message_delta: Message delta (contains usage)/g' \
  -e 's/- message_stop: 消息结束/- message_stop: Message end/g' \
  -e 's/Usestandard SSE Parse器/Use standard SSE parser/g' \
  -e 's/共享状态：保存 message_id/Shared state: save message_id/g' \
  -e 's/Convert事件流/Convert event stream/g' \
  -e 's/Parse Anthropic streaming事件/Parse Anthropic streaming event/g' \
  -e 's/提取并保存 message id/Extract and save message id/g' \
  -e 's/message_start 不Returnscontent/message_start does not return content/g' \
  -e 's/提取文本delta/Extract text delta/g' \
  -e 's/构造 StreamingResponse/Construct StreamingResponse/g' \
  -e 's/提取 usage and stop_reason/Extract usage and stop_reason/g' \
  -e 's/Returns最终response（Contains finish_reason and usage）/Return final response (contains finish_reason and usage)/g' \
  -e 's/忽略其他事件type/Ignore other event types/g' \
  -e 's/OpenAIprotocolimplementation - V2架构/OpenAI Protocol Implementation - V2 Architecture/g' \
  -e 's/this模chunkimplementationstandardOpenAI APIprotocol规范。/This module implements the standard OpenAI API protocol specification./g' \
  -e 's/Create新OpenAIProtocol instance/Create new OpenAI Protocol instance/g' \
  -e 's/纯文本：Use字符串format/Plain text: use string format/g' \
  -e 's/多模态：Use数组format/Multi-modal: use array format/g' \
  -e 's/纯文本/Plain text/g' \
  -e 's/多模态数组/Multi-modal array/g' \
  -e 's/提取第a选择content作as便利字段（纯文本）/Extract first choice content as convenience field (plain text)/g' \
  -e 's/提取第achoicereasoning_content/Extract first choice reasoning_content/g' \
  {} \;

echo "All Chinese characters removed\!"
