#!/bin/bash

# 测试腾讯云混元 API 原始响应

export TENCENT_API_KEY=sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50

echo "🧪 Testing Tencent Hunyuan API Raw Response"
echo "============================================"
echo ""

echo "📋 Sending request to Tencent Hunyuan API..."
echo ""

curl -s -X POST https://api.hunyuan.cloud.tencent.com/v1/chat/completions \
  -H "Authorization: Bearer $TENCENT_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "hunyuan-lite",
    "messages": [{"role": "user", "content": "你好"}],
    "max_tokens": 1000
  }' | jq '.' | tee /tmp/tencent-response.json

echo ""
echo ""
echo "📊 Analysis:"
echo "============"
echo ""

if [ -f /tmp/tencent-response.json ]; then
    echo "Response structure:"
    echo ""
    
    # 检查是否有错误
    if jq -e '.error' /tmp/tencent-response.json > /dev/null 2>&1; then
        echo "❌ Error response:"
        jq '.error' /tmp/tencent-response.json
        echo ""
        echo "常见错误:"
        echo "  - invalid_api_key: API Key 无效"
        echo "  - model_not_found: 模型不存在"
        echo "  - invalid_request_error: 请求格式错误"
    else
        echo "✅ Success response"
        
        # 检查 choices
        if jq -e '.choices' /tmp/tencent-response.json > /dev/null 2>&1; then
            echo "✅ Has 'choices' field"
            echo ""
            echo "Content:"
            jq -r '.choices[0].message.content' /tmp/tencent-response.json
        fi
        
        # 检查 usage
        if jq -e '.usage' /tmp/tencent-response.json > /dev/null 2>&1; then
            echo ""
            echo "✅ Has 'usage' field"
            jq '.usage' /tmp/tencent-response.json
        fi
    fi
    
    echo ""
    echo "Full response saved to: /tmp/tencent-response.json"
else
    echo "❌ Failed to get response"
fi

echo ""

