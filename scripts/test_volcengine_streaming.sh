#!/bin/bash
# Volcengine Streaming 测试脚本
# 用于验证 Volcengine Doubao-Seed-Code 推理模型的流式响应修复

set -e

echo "🧪 Volcengine Streaming 测试"
echo "=============================="
echo ""

# 检查参数
if [ $# -lt 2 ]; then
    echo "用法: $0 <api-key> <endpoint>"
    echo ""
    echo "示例:"
    echo "  $0 xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx ep-20250118155555-xxxxx"
    echo ""
    exit 1
fi

API_KEY="$1"
ENDPOINT="$2"

echo "📋 测试配置:"
echo "  API Key: ${API_KEY:0:8}...${API_KEY: -4}"
echo "  Endpoint: $ENDPOINT"
echo ""

# 运行测试
echo "🚀 运行 Volcengine streaming 测试..."
echo ""

cargo run --example volcengine_streaming --features streaming -- "$API_KEY" "$ENDPOINT" 2>&1 | tee /tmp/volcengine_test.log

echo ""
echo "=============================="
echo "📊 测试结果分析"
echo "=============================="
echo ""

# 检查关键指标
TOTAL_CHUNKS=$(grep "Total chunks:" /tmp/volcengine_test.log | awk '{print $3}')
CONTENT_LENGTH=$(grep "Total content length:" /tmp/volcengine_test.log | awk '{print $4}')

echo "✅ 关键指标:"
echo "  - Total chunks: $TOTAL_CHUNKS"
echo "  - Content length: $CONTENT_LENGTH chars"
echo ""

# 验证结果
if [ "$TOTAL_CHUNKS" -gt 0 ] && [ "$CONTENT_LENGTH" -gt 0 ]; then
    echo "✅ 测试通过！"
    echo ""
    echo "修复验证成功："
    echo "  ✓ 流式响应正常接收 ($TOTAL_CHUNKS chunks)"
    echo "  ✓ 内容正确提取 ($CONTENT_LENGTH chars)"
    echo "  ✓ get_content() 返回非空值"
    echo ""
    exit 0
else
    echo "❌ 测试失败！"
    echo ""
    echo "问题诊断："
    if [ "$TOTAL_CHUNKS" -eq 0 ]; then
        echo "  ✗ 未收到流式响应 chunks"
    fi
    if [ "$CONTENT_LENGTH" -eq 0 ]; then
        echo "  ✗ 内容提取失败（get_content() 返回 None）"
    fi
    echo ""
    echo "请检查："
    echo "  1. API Key 是否正确"
    echo "  2. Endpoint ID 是否有效"
    echo "  3. 网络连接是否正常"
    echo ""
    exit 1
fi

