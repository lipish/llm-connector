#!/bin/bash

# 测试火山引擎 API 原始响应

export VOLCENGINE_API_KEY=26f962bd-450e-4876-bc32-a732e6da9cd2

echo "🧪 Testing Volcengine API Raw Response"
echo "======================================="
echo ""

echo "📋 Sending request to Volcengine API..."
echo ""
echo "注意: 需要替换 model 为实际的端点 ID (ep-xxxxxx)"
echo ""

# 使用实际的端点 ID
curl -s -X POST https://ark.cn-beijing.volces.com/api/v3/chat/completions \
  -H "Authorization: Bearer $VOLCENGINE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ep-20251006132256-vrq2p",
    "messages": [{"role": "user", "content": "你好"}],
    "max_tokens": 1000
  }' | jq '.' | tee /tmp/volcengine-response.json

echo ""
echo ""
echo "📊 Analysis:"
echo "============"
echo ""

if [ -f /tmp/volcengine-response.json ]; then
    echo "Response structure:"
    echo ""
    
    # 检查是否有错误
    if jq -e '.error' /tmp/volcengine-response.json > /dev/null 2>&1; then
        echo "❌ Error response:"
        jq '.error' /tmp/volcengine-response.json
        echo ""
        echo "常见错误:"
        echo "  - invalid_api_key: API Key 无效"
        echo "  - model_not_found: 端点 ID 不存在或无权访问"
        echo "  - invalid_request_error: 请求格式错误"
        echo ""
        echo "解决方法:"
        echo "  1. 检查 API Key 是否正确"
        echo "  2. 检查端点 ID 是否正确（格式: ep-xxxxxx）"
        echo "  3. 在火山引擎控制台获取端点 ID:"
        echo "     https://console.volcengine.com/ark/region:ark+cn-beijing/endpoint/"
    else
        echo "✅ Success response"
        
        # 检查 choices
        if jq -e '.choices' /tmp/volcengine-response.json > /dev/null 2>&1; then
            echo "✅ Has 'choices' field"
            echo ""
            echo "Content:"
            jq -r '.choices[0].message.content' /tmp/volcengine-response.json
        fi
        
        # 检查 usage
        if jq -e '.usage' /tmp/volcengine-response.json > /dev/null 2>&1; then
            echo ""
            echo "✅ Has 'usage' field"
            jq '.usage' /tmp/volcengine-response.json
        fi
    fi
    
    echo ""
    echo "Full response saved to: /tmp/volcengine-response.json"
else
    echo "❌ Failed to get response"
fi

echo ""

