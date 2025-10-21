#!/bin/bash

# 批量修复示例文件中的 Message 构造

set -e

echo "🔧 批量修复示例文件..."

# 修复模式 1: content: "...".to_string() -> Message::text()
# 这个比较复杂，需要手动处理

# 修复模式 2: message.content -> message.content_as_text()
# 在 println! 等宏中使用

echo "📝 修复 println! 中的 message.content..."

find examples -name "*.rs" -type f | while read file; do
    # 备份
    cp "$file" "$file.bak"
    
    # 替换 message.content 为 message.content_as_text()
    # 但只在 println!, print!, format! 等宏中
    sed -i '' 's/\.content)/\.content_as_text())/g' "$file"
    sed -i '' 's/\[0\]\.content/[0].content_as_text()/g' "$file"
    sed -i '' 's/msg\.content/msg.content_as_text()/g' "$file"
    sed -i '' 's/choice\.message\.content/choice.message.content_as_text()/g' "$file"
    
    echo "  ✅ $file"
done

echo ""
echo "✅ 批量修复完成！"
echo ""
echo "⚠️  注意: 以下模式需要手动修复:"
echo "   Message { role: Role::User, content: \"...\".to_string(), ... }"
echo "   改为:"
echo "   Message::text(Role::User, \"...\")"
echo ""
echo "🔍 查找需要手动修复的文件:"
cargo build --examples 2>&1 | grep "error\[E0308\]" | grep "examples/" | cut -d':' -f1 | sort -u

