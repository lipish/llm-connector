#!/bin/bash

# 测试腾讯云混元流式响应

export TENCENT_API_KEY=sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50

echo "🧪 Testing Tencent Hunyuan Streaming Response"
echo "=============================================="
echo ""

echo "📋 Sending streaming request to Tencent Hunyuan API..."
echo ""

curl -s -N -X POST https://api.hunyuan.cloud.tencent.com/v1/chat/completions \
  -H "Authorization: Bearer $TENCENT_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "hunyuan-lite",
    "messages": [{"role": "user", "content": "用一句话介绍北京"}],
    "max_tokens": 1000,
    "stream": true
  }' 2>&1 | tee /tmp/tencent-streaming.txt

echo ""
echo ""
echo "📊 Analysis:"
echo "============"
echo ""

if [ -f /tmp/tencent-streaming.txt ]; then
    echo "Response format:"
    echo ""
    
    # 检查是否是 SSE 格式
    if grep -q "^data:" /tmp/tencent-streaming.txt; then
        echo "✅ SSE format detected (data: prefix)"
        echo ""
        echo "Number of data chunks:"
        grep -c "^data:" /tmp/tencent-streaming.txt
        echo ""
        echo "First few chunks:"
        grep "^data:" /tmp/tencent-streaming.txt | head -10
        echo ""
        
        # 检查是否有 [DONE]
        if grep -q "data: \[DONE\]" /tmp/tencent-streaming.txt; then
            echo "✅ Has [DONE] marker"
        fi
    else
        echo "❌ Not SSE format"
        echo ""
        echo "Response preview:"
        head -20 /tmp/tencent-streaming.txt
        echo ""
    fi
    
    echo "Full response saved to: /tmp/tencent-streaming.txt"
else
    echo "❌ Failed to get response"
fi

echo ""

