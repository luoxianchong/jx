#!/bin/bash

# jx 安装脚本
# 用于将jx工具安装到系统路径

set -e

echo "🚀 开始安装 jx - Fast Java Package Manager"
echo ""

# 检查Rust是否安装
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到Rust/Cargo，请先安装Rust"
    echo "   访问 https://rustup.rs/ 安装Rust"
    exit 1
fi

echo "✅ 检测到Rust版本: $(rustc --version)"
echo ""

# 构建项目
echo "🔨 正在构建jx..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ 构建失败"
    exit 1
fi

echo "✅ 构建完成"
echo ""

# 确定安装路径
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# 安装jx
echo "📦 正在安装jx到 $INSTALL_DIR..."
sudo cp target/release/jx "$INSTALL_DIR/"

if [ $? -ne 0 ]; then
    echo "❌ 安装失败，尝试不使用sudo..."
    cp target/release/jx "$INSTALL_DIR/"
fi

# 设置执行权限
chmod +x "$INSTALL_DIR/jx"

echo "✅ 安装完成"
echo ""

# 检查PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  警告: $INSTALL_DIR 不在PATH中"
    echo "   请将以下行添加到您的shell配置文件 (.bashrc, .zshrc 等):"
    echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
fi

# 测试安装
if command -v jx &> /dev/null; then
    echo "🎉 jx安装成功!"
    echo "   版本: $(jx --version)"
    echo ""
    echo "使用方法:"
    echo "  jx init my-project --template maven    # 创建Maven项目"
    echo "  jx init my-project --template gradle   # 创建Gradle项目"
    echo "  jx install                              # 安装依赖"
    echo "  jx add org.example:lib:1.0.0           # 添加依赖"
    echo "  jx --help                               # 查看所有命令"
else
    echo "❌ 安装可能失败，jx命令不可用"
    echo "   请检查PATH设置或手动安装"
fi
