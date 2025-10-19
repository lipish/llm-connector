#!/bin/bash

# 测试阿里云 DashScope API 的实际响应格式

export ALIYUN_API_KEY=sk-17cb8a1feec2440bad2c5a73d7d08af2

echo "🧪 Testing Aliyun DashScope API Response Format"
echo "================================================"
echo ""

echo "📋 Sending request to Aliyun DashScope API..."
echo ""

curl -s -X POST https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation \
  -H "Authorization: Bearer $ALIYUN_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen-turbo",
    "input": {
      "messages": [
        {"role": "user", "content": "你好"}
      ]
    },
    "parameters": {
      "result_format": "message"
    }
  }' | jq '.' | tee /tmp/aliyun-response.json

echo ""
echo ""
echo "📊 Analysis:"
echo "============"
echo ""

if [ -f /tmp/aliyun-response.json ]; then
    echo "Response structure:"
    echo ""
    
    # 检查 output 字段
    if jq -e '.output' /tmp/aliyun-response.json > /dev/null 2>&1; then
        echo "✅ Has 'output' field"
        
        # 检查 output.choices
        if jq -e '.output.choices' /tmp/aliyun-response.json > /dev/null 2>&1; then
            echo "✅ Has 'output.choices' field"
            
            # 检查 choices[0].message
            if jq -e '.output.choices[0].message' /tmp/aliyun-response.json > /dev/null 2>&1; then
                echo "✅ Has 'output.choices[0].message' field"
                
                # 检查 message.content
                if jq -e '.output.choices[0].message.content' /tmp/aliyun-response.json > /dev/null 2>&1; then
                    echo "✅ Has 'output.choices[0].message.content' field"
                    echo ""
                    echo "Content:"
                    jq -r '.output.choices[0].message.content' /tmp/aliyun-response.json
                else
                    echo "❌ Missing 'output.choices[0].message.content' field"
                fi
            else
                echo "❌ Missing 'output.choices[0].message' field"
            fi
        else
            echo "❌ Missing 'output.choices' field"
            
            # 检查是否有 output.text
            if jq -e '.output.text' /tmp/aliyun-response.json > /dev/null 2>&1; then
                echo "⚠️  Found 'output.text' instead (wrong result_format?)"
                echo ""
                echo "Text:"
                jq -r '.output.text' /tmp/aliyun-response.json
            fi
        fi
    else
        echo "❌ Missing 'output' field"
    fi
    
    echo ""
    echo "Full response saved to: /tmp/aliyun-response.json"
else
    echo "❌ Failed to get response"
fi

echo ""

