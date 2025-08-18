#!/bin/bash

# jx 演示脚本
# 展示jx工具的主要功能

set -e

echo "🎬 jx 工具演示"
echo "================"
echo ""

# 检查jx是否可用
if ! command -v jx &> /dev/null; then
    echo "❌ jx工具不可用，请先运行 ./install.sh 安装"
    exit 1
fi

echo "✅ jx工具可用，版本: $(jx --version)"
echo ""

# 创建演示目录
DEMO_DIR="jx-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

echo "📁 创建演示目录: $DEMO_DIR"
echo ""

# 演示1: 创建Maven项目
echo "🔧 演示1: 创建Maven项目"
echo "------------------------"
jx init maven-demo --template maven
echo ""

# 演示2: 创建Gradle项目
echo "🔧 演示2: 创建Gradle项目"
echo "------------------------"
jx init gradle-demo --template gradle
echo ""

# 演示3: 添加依赖
echo "➕ 演示3: 添加依赖到Maven项目"
echo "------------------------------"
cd maven-demo
jx add org.apache.commons:commons-lang3:3.12.0 --scope compile
echo ""

# 演示4: 安装依赖
echo "📦 演示4: 安装依赖"
echo "------------------"
jx install
echo ""

# 显示项目结构
echo "📋 项目结构:"
echo "-------------"
cd ..
tree -I 'target|.gradle|build' 2>/dev/null || find . -type f | grep -E '\.(java|xml|gradle)$' | sort

echo ""
echo "🎉 演示完成!"
echo ""
echo "您可以:"
echo "  cd maven-demo && jx install    # 在Maven项目中安装依赖"
echo "  cd gradle-demo && jx install   # 在Gradle项目中安装依赖"
echo "  jx --help                      # 查看所有可用命令"
echo ""
echo "演示目录: $DEMO_DIR"
