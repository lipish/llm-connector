#!/bin/bash

# 测试阿里云 DashScope API 的流式响应格式

export ALIYUN_API_KEY=sk-17cb8a1feec2440bad2c5a73d7d08af2

echo "🧪 Testing Aliyun DashScope Streaming Response Format"
echo "======================================================"
echo ""

echo "📋 Sending streaming request to Aliyun DashScope API..."
echo ""

curl -s -N -X POST https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation \
  -H "Authorization: Bearer $ALIYUN_API_KEY" \
  -H "Content-Type: application/json" \
  -H "X-DashScope-SSE: enable" \
  -d '{
    "model": "qwen-turbo",
    "input": {
      "messages": [
        {"role": "user", "content": "用一句话介绍北京"}
      ]
    },
    "parameters": {
      "result_format": "message",
      "incremental_output": true
    }
  }' 2>&1 | tee /tmp/aliyun-streaming.txt

echo ""
echo ""
echo "📊 Analysis:"
echo "============"
echo ""

if [ -f /tmp/aliyun-streaming.txt ]; then
    echo "Response format:"
    echo ""
    
    # 检查是否是 SSE 格式
    if grep -q "^data:" /tmp/aliyun-streaming.txt; then
        echo "✅ SSE format detected (data: prefix)"
        echo ""
        echo "Number of data chunks:"
        grep -c "^data:" /tmp/aliyun-streaming.txt
        echo ""
        echo "First few chunks:"
        grep "^data:" /tmp/aliyun-streaming.txt | head -5
        echo ""
    else
        echo "❌ Not SSE format"
        echo ""
        echo "Response preview:"
        head -20 /tmp/aliyun-streaming.txt
        echo ""
    fi
    
    # 检查是否有 incremental_output
    if grep -q "incremental_output" /tmp/aliyun-streaming.txt; then
        echo "⚠️  Response mentions incremental_output"
    fi
    
    echo "Full response saved to: /tmp/aliyun-streaming.txt"
else
    echo "❌ Failed to get response"
fi

echo ""

