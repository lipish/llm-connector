#!/bin/bash

# 测试火山引擎流式响应

export VOLCENGINE_API_KEY=26f962bd-450e-4876-bc32-a732e6da9cd2

echo "🧪 Testing Volcengine Streaming Response"
echo "========================================="
echo ""

echo "📋 Sending streaming request to Volcengine API..."
echo ""

curl -s -N -X POST https://ark.cn-beijing.volces.com/api/v3/chat/completions \
  -H "Authorization: Bearer $VOLCENGINE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ep-20251006132256-vrq2p",
    "messages": [{"role": "user", "content": "用一句话介绍北京"}],
    "max_tokens": 1000,
    "stream": true
  }' 2>&1 | tee /tmp/volcengine-streaming.txt

echo ""
echo ""
echo "📊 Analysis:"
echo "============"
echo ""

if [ -f /tmp/volcengine-streaming.txt ]; then
    echo "Response format:"
    echo ""
    
    # 检查是否是 SSE 格式
    if grep -q "^data:" /tmp/volcengine-streaming.txt; then
        echo "✅ SSE format detected (data: prefix)"
        echo ""
        echo "Number of data chunks:"
        grep -c "^data:" /tmp/volcengine-streaming.txt
        echo ""
        echo "First few chunks:"
        grep "^data:" /tmp/volcengine-streaming.txt | head -10
        echo ""
        
        # 检查是否有 [DONE]
        if grep -q "data: \[DONE\]" /tmp/volcengine-streaming.txt; then
            echo "✅ Has [DONE] marker"
        fi
    else
        echo "❌ Not SSE format"
        echo ""
        echo "Response preview:"
        head -20 /tmp/volcengine-streaming.txt
        echo ""
    fi
    
    echo "Full response saved to: /tmp/volcengine-streaming.txt"
else
    echo "❌ Failed to get response"
fi

echo ""

