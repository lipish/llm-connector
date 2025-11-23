#!/bin/bash
# Fix remaining Chinese comments - Part 2

cd /Users/mac-m4/github/llm-connector

echo "Fixing remaining Chinese comments..."

# Fix all remaining patterns
find src -name "*.rs" -type f -exec sed -i '' \
  -e 's/Create新 Provider Builder/Create new Provider Builder/g' \
  -e 's/if没有content增量，清空 delta.content/If no content delta, clear delta.content/g' \
  -e 's/智谱专用data结构 (OpenAI兼容格式)/Zhipu-specific data structure (OpenAI compatible format)/g' \
  -e 's/Zhipu GLM服务Provide商类型/Zhipu GLM service provider type/g' \
  -e 's/CreateZhipu GLM服务Provide商 (Use原生格式)/Create Zhipu GLM service provider (using native format)/g' \
  -e 's/configuration好智谱服务Provide商instance/Configured Zhipu service provider instance/g' \
  -e 's/CreateZhipu GLM服务Provide商 (UseOpenAI兼容格式)/Create Zhipu GLM service provider (using OpenAI compatible format)/g' \
  -e 's/configuration好智谱服务Provide商instance (OpenAI兼容模式)/Configured Zhipu service provider instance (OpenAI compatible mode)/g' \
  -e 's/Create带有customconfigurationZhipu GLM服务Provide商/Create Zhipu GLM service provider with custom configuration/g' \
  -e 's/超时时间(秒) (optional)/Timeout (seconds) (optional)/g' \
  -e 's/代理URL (optional)/Proxy URL (optional)/g' \
  -e 's/UseOpenAI兼容格式/Use OpenAI compatible format/g' \
  -e 's/Use默认URL/Use default URL/g' \
  -e 's/添加authentication头/Add authentication headers/g' \
  -e 's/Create带有custom超时Zhipu GLM服务Provide商/Create Zhipu GLM service provider with custom timeout/g' \
  -e 's/超时时间(秒)/Timeout (seconds)/g' \
  -e 's/Set120秒超时/Set 120 seconds timeout/g' \
  -e 's/CreateforZhipu GLM企业版服务Provide商/Create Zhipu GLM enterprise service provider/g' \
  -e 's/企业版API key/Enterprise API key/g' \
  -e 's/企业版endpointURL/Enterprise endpoint URL/g' \
  -e 's/ValidateZhipu GLM API key格式/Validate Zhipu GLM API key format/g' \
  -e 's/要ValidateAPI key/API key to validate/g' \
  -e 's/if格式看起to正确Returnstrue，if则Returnsfalse/Returns true if format looks correct, otherwise returns false/g' \
  -e 's/测试Contains推理content情况/Test case with reasoning content/g' \
  -e 's/测试不Contains推理content情况/Test case without reasoning content/g' \
  -e 's/测试只有 Thinking 没有 Response 情况/Test case with only Thinking, no Response/g' \
  -e 's/测试空推理content情况/Test case with empty reasoning content/g' \
  -e 's/测试推理modelstreamingresponse/Test reasoning model streaming response/g' \
  -e 's/第a块:/First chunk:/g' \
  -e 's/第二个块:/Second chunk:/g' \
  -e 's/第三个块:/Third chunk:/g' \
  -e 's/第四个块:/Fourth chunk:/g' \
  -e 's/推理过程/Reasoning process/g' \
  -e 's/继续答案/Continue answer/g' \
  -e 's/测试非推理modelstreamingresponse/Test non-reasoning model streaming response/g' \
  -e 's/普通content/Normal content/g' \
  -e 's/继续content/Continue content/g' \
  -e 's/测试完整推理ina块中/Test complete reasoning in one chunk/g' \
  -e 's/LongCat API 服务Provide商实现/LongCat API Service Provider Implementation/g' \
  -e 's/LongCat Support两种 API 格式：/LongCat supports two API formats:/g' \
  -e 's/OpenAI 格式 - Use OpenAI 兼容接口/OpenAI format - Uses OpenAI compatible interface/g' \
  -e 's/Anthropic 格式 - Use Anthropic 兼容接口，但authentication方式as Bearer token/Anthropic format - Uses Anthropic compatible interface, but with Bearer token authentication/g' \
  -e 's/Note：LongCat  Anthropic 格式Use `Authorization: Bearer` authentication，/Note: LongCat Anthropic format uses `Authorization: Bearer` authentication,/g' \
  -e 's/而不is标准 Anthropic  `x-api-key` authentication。/instead of standard Anthropic `x-api-key` authentication./g' \
  -e 's/LongCat Anthropic 格式protocoladapter/LongCat Anthropic format protocol adapter/g' \
  -e 's/Use ConfigurableProtocol 包装 Anthropic protocol，Use Bearer authentication/Uses ConfigurableProtocol to wrap Anthropic protocol, using Bearer authentication/g' \
  -e 's/LongCat Anthropic 格式服务Provide商类型/LongCat Anthropic format service provider type/g' \
  -e 's/Create LongCat Anthropic 格式服务Provide商/Create LongCat Anthropic format service provider/g' \
  -e 's/LongCat API 密钥 (Format: ak_...)/LongCat API key (Format: ak_...)/g' \
  -e 's/Create带有customconfiguration LongCat Anthropic 服务Provide商/Create LongCat Anthropic service provider with custom configuration/g' \
  -e 's/API 密钥/API key/g' \
  -e 's/customBase URL (optional，默认as LongCat Anthropic endpoint)/Custom base URL (optional, defaults to LongCat Anthropic endpoint)/g' \
  {} \;

echo "Fixed remaining patterns"

# Fix more specific patterns
find src -name "*.rs" -type f -exec sed -i '' \
  -e 's/Provide商/Provider/g' \
  -e 's/服务/service/g' \
  -e 's/类型/type/g' \
  -e 's/实现/implementation/g' \
  -e 's/格式/format/g' \
  -e 's/兼容/compatible/g' \
  -e 's/配置/configuration/g' \
  -e 's/自定义/custom/g' \
  -e 's/默认/default/g' \
  -e 's/可选/optional/g' \
  -e 's/密钥/key/g' \
  -e 's/端点/endpoint/g' \
  -e 's/超时/timeout/g' \
  -e 's/代理/proxy/g' \
  -e 's/验证/validate/g' \
  -e 's/测试/test/g' \
  -e 's/情况/case/g' \
  -e 's/包含/contains/g' \
  -e 's/内容/content/g' \
  -e 's/推理/reasoning/g' \
  -e 's/过程/process/g' \
  -e 's/答案/answer/g' \
  -e 's/继续/continue/g' \
  -e 's/普通/normal/g' \
  -e 's/完整/complete/g' \
  -e 's/块/chunk/g' \
  -e 's/第一/first/g' \
  -e 's/第二/second/g' \
  -e 's/第三/third/g' \
  -e 's/第四/fourth/g' \
  -e 's/个/\//g' \
  -e 's/支持/support/g' \
  -e 's/两种/two/g' \
  -e 's/接口/interface/g' \
  -e 's/方式/method/g' \
  -e 's/标准/standard/g' \
  -e 's/认证/authentication/g' \
  -e 's/包装/wrap/g' \
  -e 's/使用/use/g' \
  -e 's/创建/create/g' \
  -e 's/带有/with/g' \
  -e 's/企业版/enterprise/g' \
  -e 's/返回/return/g' \
  -e 's/如果/if/g' \
  -e 's/否则/otherwise/g' \
  -e 's/正确/correct/g' \
  -e 's/看起来/looks/g' \
  -e 's/设置/set/g' \
  -e 's/秒/seconds/g' \
  -e 's/添加/add/g' \
  -e 's/头/headers/g' \
  -e 's/清空/clear/g' \
  -e 's/没有/no/g' \
  -e 's/增量/delta/g' \
  -e 's/专用/specific/g' \
  -e 's/数据结构/data structure/g' \
  -e 's/模型/model/g' \
  -e 's/流式/streaming/g' \
  -e 's/响应/response/g' \
  -e 's/只有/only/g' \
  -e 's/空/empty/g' \
  -e 's/非/non-/g' \
  -e 's/注意/Note/g' \
  -e 's/而不是/instead of/g' \
  -e 's/适配器/adapter/g' \
  -e 's/协议/protocol/g' \
  {} \;

echo "Fixed generic Chinese words"
echo "Cleanup complete!"

