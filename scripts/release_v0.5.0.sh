#!/bin/bash

# llm-connector v0.5.0 发布脚本

set -e

echo "🚀 llm-connector v0.5.0 发布流程"
echo "================================"
echo ""

# 1. 检查当前分支
echo "1️⃣ 检查当前分支..."
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo "❌ 错误: 必须在 main 分支上发布"
    echo "   当前分支: $BRANCH"
    exit 1
fi
echo "✅ 当前分支: main"
echo ""

# 2. 检查工作区状态
echo "2️⃣ 检查工作区状态..."
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ 错误: 工作区有未提交的更改"
    git status --short
    exit 1
fi
echo "✅ 工作区干净"
echo ""

# 3. 拉取最新代码
echo "3️⃣ 拉取最新代码..."
git pull origin main
echo "✅ 代码已更新"
echo ""

# 4. 运行测试
echo "4️⃣ 运行所有测试..."
cargo test --all-features
if [ $? -ne 0 ]; then
    echo "❌ 错误: 测试失败"
    exit 1
fi
echo "✅ 所有测试通过"
echo ""

# 5. 检查编译
echo "5️⃣ 检查编译..."
cargo check --all-targets --all-features
if [ $? -ne 0 ]; then
    echo "❌ 错误: 编译失败"
    exit 1
fi
echo "✅ 编译成功"
echo ""

# 6. 检查文档
echo "6️⃣ 检查文档..."
cargo doc --no-deps
if [ $? -ne 0 ]; then
    echo "❌ 错误: 文档生成失败"
    exit 1
fi
echo "✅ 文档生成成功"
echo ""

# 7. 创建 Git Tag
echo "7️⃣ 创建 Git Tag..."
TAG="v0.5.0"
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "⚠️  警告: Tag $TAG 已存在"
    read -p "是否删除并重新创建? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git tag -d "$TAG"
        git push origin ":refs/tags/$TAG"
        echo "✅ 已删除旧 Tag"
    else
        echo "❌ 取消发布"
        exit 1
    fi
fi

git tag -a "$TAG" -m "Release v0.5.0 - Native Multi-modal Content Support

## 🎉 Major Features

- Native multi-modal content support (text + images)
- MessageBlock API for flexible content composition
- Convenient constructor methods for Message
- Helper methods for content extraction

## ⚠️ Breaking Changes

- Message.content type changed from String to Vec<MessageBlock>
- Use Message::text() for simple text messages
- Use message.content_as_text() to extract text

## 🔧 Improvements

- Cleaned up 69% of example files (39 → 12)
- Cleaned up 56% of test files (18 → 8)
- Updated all examples to use new API
- Added comprehensive migration guide

## 📊 Statistics

- 221 tests passed (100% pass rate)
- 0 compilation errors
- 0 compilation warnings

See CHANGELOG.md for full details."

echo "✅ Tag $TAG 已创建"
echo ""

# 8. 推送 Tag
echo "8️⃣ 推送 Tag 到 GitHub..."
git push origin "$TAG"
echo "✅ Tag 已推送"
echo ""

# 9. 发布到 crates.io
echo "9️⃣ 发布到 crates.io..."
echo "⚠️  注意: 这将发布到 crates.io，无法撤销！"
read -p "确认发布? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cargo publish
    echo "✅ 已发布到 crates.io"
else
    echo "⚠️  跳过 crates.io 发布"
fi
echo ""

# 10. 完成
echo "================================"
echo "🎉 发布完成！"
echo ""
echo "📝 下一步:"
echo "1. 在 GitHub 上创建 Release: https://github.com/lipish/llm-connector/releases/new"
echo "   - Tag: $TAG"
echo "   - Title: llm-connector v0.5.0 - Native Multi-modal Content Support"
echo "   - 复制 docs/RELEASE_v0.5.0.md 的内容"
echo ""
echo "2. 验证 crates.io 发布: https://crates.io/crates/llm-connector"
echo ""
echo "3. 更新文档: https://docs.rs/llm-connector"
echo ""
echo "🔗 资源链接:"
echo "- GitHub: https://github.com/lipish/llm-connector"
echo "- Crates.io: https://crates.io/crates/llm-connector"
echo "- Docs.rs: https://docs.rs/llm-connector"
echo ""

