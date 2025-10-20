#!/bin/bash

# 测试 LongCat Anthropic 格式流式响应

export LONGCAT_API_KEY=ak_11o3bI6O03mx2yS8jb2hD61q7DJ4d

echo "🧪 Testing LongCat Anthropic Streaming Response"
echo "================================================"
echo ""

echo "📋 Sending streaming request to LongCat Anthropic API..."
echo ""

curl -s -N -X POST https://api.longcat.chat/anthropic/v1/messages \
  -H "Authorization: Bearer $LONGCAT_API_KEY" \
  -H "Content-Type: application/json" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "LongCat-Flash-Chat",
    "max_tokens": 1000,
    "messages": [{"role": "user", "content": "用一句话介绍北京"}],
    "stream": true
  }' 2>&1 | tee /tmp/longcat-anthropic-streaming.txt

echo ""
echo ""
echo "📊 Analysis:"
echo "============"
echo ""

if [ -f /tmp/longcat-anthropic-streaming.txt ]; then
    echo "Response format:"
    echo ""
    
    # 检查是否是 SSE 格式
    if grep -q "^event:" /tmp/longcat-anthropic-streaming.txt; then
        echo "✅ SSE format detected (event: prefix)"
        echo ""
        echo "Number of event chunks:"
        grep -c "^event:" /tmp/longcat-anthropic-streaming.txt
        echo ""
        echo "First few chunks:"
        head -30 /tmp/longcat-anthropic-streaming.txt
        echo ""
    else
        echo "❌ Not SSE format"
        echo ""
        echo "Response preview:"
        head -20 /tmp/longcat-anthropic-streaming.txt
        echo ""
    fi
    
    echo "Full response saved to: /tmp/longcat-anthropic-streaming.txt"
else
    echo "❌ Failed to get response"
fi

echo ""

