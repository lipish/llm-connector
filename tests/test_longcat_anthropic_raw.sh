#!/bin/bash

# 测试 LongCat Anthropic 格式 API 原始响应

export LONGCAT_API_KEY=ak_11o3bI6O03mx2yS8jb2hD61q7DJ4d

echo "🧪 Testing LongCat Anthropic API Raw Response"
echo "=============================================="
echo ""

echo "📋 Sending request to LongCat Anthropic API..."
echo ""

curl -s -X POST https://api.longcat.chat/anthropic/v1/messages \
  -H "Authorization: Bearer $LONGCAT_API_KEY" \
  -H "Content-Type: application/json" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "LongCat-Flash-Chat",
    "max_tokens": 1000,
    "messages": [{"role": "user", "content": "你好"}]
  }' | jq '.' | tee /tmp/longcat-anthropic-response.json

echo ""
echo ""
echo "📊 Analysis:"
echo "============"
echo ""

if [ -f /tmp/longcat-anthropic-response.json ]; then
    echo "Response structure:"
    echo ""
    
    # 检查是否有错误
    if jq -e '.error' /tmp/longcat-anthropic-response.json > /dev/null 2>&1; then
        echo "❌ Error response:"
        jq '.error' /tmp/longcat-anthropic-response.json
    else
        echo "✅ Success response"
        
        # 检查 content
        if jq -e '.content' /tmp/longcat-anthropic-response.json > /dev/null 2>&1; then
            echo "✅ Has 'content' field"
            echo ""
            echo "Content:"
            jq -r '.content[0].text' /tmp/longcat-anthropic-response.json
        fi
        
        # 检查 usage
        if jq -e '.usage' /tmp/longcat-anthropic-response.json > /dev/null 2>&1; then
            echo ""
            echo "✅ Has 'usage' field"
            jq '.usage' /tmp/longcat-anthropic-response.json
        fi
    fi
    
    echo ""
    echo "Full response saved to: /tmp/longcat-anthropic-response.json"
else
    echo "❌ Failed to get response"
fi

echo ""

