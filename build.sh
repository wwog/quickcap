#!/bin/bash

set -e  # 遇到错误立即退出

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 平台定义
WINDOWS_TARGET="x86_64-pc-windows-gnu"
MACOS_INTEL_TARGET="x86_64-apple-darwin"
MACOS_ARM_TARGET="aarch64-apple-darwin"

MACOSX_DEPLOYMENT_TARGET="12.3"

# 输出函数
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查命令是否存在
check_command() {
    if ! command -v "$1" &> /dev/null; then
        return 1
    fi
    return 0
}

# 检查 Rust 工具链
check_rust() {
    info "检查 Rust 工具链..."
    if ! check_command rustc; then
        error "未找到 rustc，请先安装 Rust: https://rustup.rs/"
        exit 1
    fi
    
    if ! check_command cargo; then
        error "未找到 cargo，请先安装 Rust: https://rustup.rs/"
        exit 1
    fi
    
    local rust_version=$(rustc --version)
    info "Rust 版本: $rust_version"
}

# 检查并安装 Rust target
check_rust_target() {
    local target=$1
    info "检查 Rust target: $target"
    
    if rustup target list --installed | grep -q "^${target}$"; then
        info "Target $target 已安装"
    else
        warn "Target $target 未安装，正在安装..."
        rustup target add "$target"
    fi
}

# 检查 macOS 构建依赖
check_macos_dependencies() {
    info "检查 macOS 构建依赖..."
    
    # 检查 Xcode 命令行工具
    if ! check_command xcode-select; then
        error "未找到 Xcode 命令行工具，请运行: xcode-select --install"
        exit 1
    fi
    
    # 检查 clang
    if ! check_command clang; then
        error "未找到 clang，请安装 Xcode 命令行工具: xcode-select --install"
        exit 1
    fi
    
    # 检查 SDK
    local sdk_path=$(xcrun --show-sdk-path 2>/dev/null || echo "")
    if [ -z "$sdk_path" ]; then
        error "无法找到 macOS SDK，请确保 Xcode 已正确安装"
        exit 1
    fi
    info "macOS SDK 路径: $sdk_path"
    
    # 检查 Swift（capscreen_macos 可能需要）
    if check_command swiftc; then
        local swift_version=$(swiftc -version | head -n1)
        info "Swift 版本: $swift_version"
    else
        warn "未找到 Swift 编译器，某些依赖可能需要 Swift"
    fi
}

# 检查 Windows 交叉编译依赖 (GNU工具链)
check_windows_dependencies() {
    info "检查 Windows 交叉编译依赖 (GNU工具链)..."
    
    # 检查 mingw-w64
    local mingw_found=false
    
    # 检查 Homebrew 安装的 mingw-w64
    if check_command x86_64-w64-mingw32-gcc || check_command x86_64-w64-mingw32-g++; then
        mingw_found=true
        info "找到 mingw-w64 (通过 Homebrew)"
    fi
    
    # 检查 MacPorts 安装的 mingw-w64
    if [ -f "/opt/local/bin/x86_64-w64-mingw32-gcc" ]; then
        mingw_found=true
        info "找到 mingw-w64 (通过 MacPorts)"
    fi
    
    if [ "$mingw_found" = false ]; then
        warn "未找到 mingw-w64，正在尝试安装..."
        
        if check_command brew; then
            info "使用 Homebrew 安装 mingw-w64..."
            brew install mingw-w64 || {
                error "安装 mingw-w64 失败，请手动安装: brew install mingw-w64"
                exit 1
            }
        else
            error "未找到 Homebrew，请手动安装 mingw-w64:"
            error "  brew install mingw-w64"
            error "或使用 MacPorts:"
            error "  sudo port install mingw-w64"
            exit 1
        fi
    fi
    
    # 验证 mingw-w64 工具
    if check_command x86_64-w64-mingw32-gcc; then
        local gcc_version=$(x86_64-w64-mingw32-gcc --version | head -n1)
        info "mingw-w64 GCC 版本: $gcc_version"
    else
        error "无法找到 x86_64-w64-mingw32-gcc"
        exit 1
    fi
    
    # 配置 Cargo 使用 mingw-w64
    setup_windows_cargo_config
}

# 设置 Windows 交叉编译的 Cargo 配置
setup_windows_cargo_config() {
    local cargo_config_dir=".cargo"
    local cargo_config_file="$cargo_config_dir/config.toml"
    
    # 查找 mingw-w64 的路径
    local mingw_gcc=$(which x86_64-w64-mingw32-gcc 2>/dev/null || echo "")
    if [ -z "$mingw_gcc" ]; then
        error "无法找到 mingw-w64 GCC"
        exit 1
    fi
    
    local mingw_bin_dir=$(dirname "$mingw_gcc")
    
    info "配置 Cargo 使用 mingw-w64..."
    mkdir -p "$cargo_config_dir"
    
    # 创建或更新 config.toml
    if [ -f "$cargo_config_file" ]; then
        # 检查是否已配置
        if grep -q "target.x86_64-pc-windows-gnu" "$cargo_config_file"; then
            info "Cargo 配置已存在 Windows GNU target 配置"
        else
            info "追加 Windows GNU target 配置到现有 config.toml"
            cat >> "$cargo_config_file" << EOF

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
EOF
        fi
    else
        info "创建新的 Cargo config.toml"
        cat > "$cargo_config_file" << EOF
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
EOF
    fi
    
    # 设置 PATH 以确保能找到工具
    export PATH="$mingw_bin_dir:$PATH"
}

# 构建 Windows x64
build_windows() {
    info "开始构建 Windows x64 (GNU工具链)..."
    
    check_rust_target "$WINDOWS_TARGET"
    check_windows_dependencies
    
    export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
    
    info "构建目标: $WINDOWS_TARGET"
    cargo build --release --target "$WINDOWS_TARGET" || {
        error "Windows 构建失败"
        exit 1
    }
    
    local output_file="target/${WINDOWS_TARGET}/release/quickcap.exe"
    if [ -f "$output_file" ]; then
        info "构建成功: $output_file"
        # 打包并移动到 dist 目录
        local dist_dir="dist"
        mkdir -p "$dist_dir"
        local archive="${dist_dir}/quickcap-windows-x64.tar.gz"
        info "打包 Windows 可执行文件到: $archive"
        tar -czf "$archive" -C "target/${WINDOWS_TARGET}/release" "quickcap.exe"
    else
        error "构建产物不存在: $output_file"
        exit 1
    fi
}

# 构建 macOS Intel (x86_64)
build_macos_intel() {
    info "开始构建 macOS Intel (x86_64)..."
    
    check_rust_target "$MACOS_INTEL_TARGET"
    check_macos_dependencies
    
    local current_arch=$(uname -m)
    
    # 设置交叉编译环境变量
    export MACOSX_DEPLOYMENT_TARGET="$MACOSX_DEPLOYMENT_TARGET"
    export ARCHS="x86_64"
    export ARCH="x86_64"
    export TARGET_ARCH="x86_64"
    export SWIFT_TARGET_TRIPLE="x86_64-apple-macosx${MACOSX_DEPLOYMENT_TARGET}"
    export CC_x86_64_apple_darwin="clang -arch x86_64 -mmacosx-version-min=${MACOSX_DEPLOYMENT_TARGET}"
    export CXX_x86_64_apple_darwin="clang++ -arch x86_64 -mmacosx-version-min=${MACOSX_DEPLOYMENT_TARGET}"
    
    # Swift 编译器参数
    local sdk_path=$(xcrun --show-sdk-path)
    export SWIFTFLAGS="-target ${SWIFT_TARGET_TRIPLE} -sdk ${sdk_path} -arch x86_64"
    export SWIFT_ARCHS="x86_64"
    export SWIFT_BUILD_ARCH="x86_64"
    export SWIFT_PLATFORM_PREFERRED_ARCH="x86_64"
    
    if [ "$current_arch" = "arm64" ]; then
        info "在 ARM Mac 上交叉编译 x86_64 macOS"
        info "使用编译器: $CC_x86_64_apple_darwin"
        info "Swift 目标: $SWIFT_TARGET_TRIPLE"
    else
        info "在 Intel Mac 上原生构建 x86_64 macOS"
    fi
    
    # 清理之前的构建缓存（如果需要）
    if [ -d "target/${MACOS_INTEL_TARGET}" ]; then
        warn "清理之前的构建缓存..."
        cargo clean -p capscreen_macos 2>/dev/null || true
    fi
    
    info "构建目标: $MACOS_INTEL_TARGET"
    cargo build --release --target "$MACOS_INTEL_TARGET" || {
        error "macOS Intel 构建失败"
        exit 1
    }
    
    local output_file="target/${MACOS_INTEL_TARGET}/release/quickcap"
    if [ -f "$output_file" ]; then
        chmod +x "$output_file"
        info "构建成功: $output_file"
        # 打包并移动到 dist 目录
        local dist_dir="dist"
        mkdir -p "$dist_dir"
        local archive="${dist_dir}/quickcap-macos-intel.tar.gz"
        info "打包 macOS Intel 可执行文件到: $archive"
        tar -czf "$archive" -C "target/${MACOS_INTEL_TARGET}/release" "quickcap"
    else
        error "构建产物不存在: $output_file"
        exit 1
    fi
}

# 构建 macOS ARM (Apple Silicon)
build_macos_arm() {
    info "开始构建 macOS ARM (Apple Silicon)..."
    
    check_rust_target "$MACOS_ARM_TARGET"
    check_macos_dependencies
    
    export MACOSX_DEPLOYMENT_TARGET="$MACOSX_DEPLOYMENT_TARGET"
    
    local current_arch=$(uname -m)
    if [ "$current_arch" = "arm64" ]; then
        info "在 ARM Mac 上原生构建"
    else
        warn "在 Intel Mac 上构建 ARM64，可能需要 Rosetta 2 或交叉编译工具"
    fi
    
    info "构建目标: $MACOS_ARM_TARGET"
    cargo build --release --target "$MACOS_ARM_TARGET" || {
        error "macOS ARM 构建失败"
        exit 1
    }
    
    local output_file="target/${MACOS_ARM_TARGET}/release/quickcap"
    if [ -f "$output_file" ]; then
        chmod +x "$output_file"
        info "构建成功: $output_file"
        # 打包并移动到 dist 目录
        local dist_dir="dist"
        mkdir -p "$dist_dir"
        local archive="${dist_dir}/quickcap-macos-arm.tar.gz"
        info "打包 macOS ARM 可执行文件到: $archive"
        tar -czf "$archive" -C "target/${MACOS_ARM_TARGET}/release" "quickcap"
    else
        error "构建产物不存在: $output_file"
        exit 1
    fi
}

# 主函数
main() {
    info "开始构建流程..."
    info "当前系统: $(uname -s) $(uname -m)"
    
    # 检查基本依赖
    check_rust
    
    # 解析命令行参数
    local build_windows_flag=false
    local build_macos_intel_flag=false
    local build_macos_arm_flag=false
    
    if [ $# -eq 0 ]; then
        # 没有参数，构建所有平台
        build_windows_flag=true
        build_macos_intel_flag=true
        build_macos_arm_flag=true
    else
        # 解析参数
        while [[ $# -gt 0 ]]; do
            case $1 in
                --windows|-w)
                    build_windows_flag=true
                    shift
                    ;;
                --macos-intel|-i)
                    build_macos_intel_flag=true
                    shift
                    ;;
                --macos-arm|-a)
                    build_macos_arm_flag=true
                    shift
                    ;;
                --all|-A)
                    build_windows_flag=true
                    build_macos_intel_flag=true
                    build_macos_arm_flag=true
                    shift
                    ;;
                --help|-h)
                    echo "用法: $0 [选项]"
                    echo ""
                    echo "选项:"
                    echo "  --windows, -w      构建 Windows x64"
                    echo "  --macos-intel, -i  构建 macOS Intel (x86_64)"
                    echo "  --macos-arm, -a    构建 macOS ARM (Apple Silicon)"
                    echo "  --all, -A          构建所有平台（默认）"
                    echo "  --help, -h         显示此帮助信息"
                    echo ""
                    echo "示例:"
                    echo "  $0                 # 构建所有平台"
                    echo "  $0 --windows      # 只构建 Windows"
                    echo "  $0 -w -a          # 构建 Windows 和 macOS ARM"
                    exit 0
                    ;;
                *)
                    error "未知选项: $1"
                    echo "使用 --help 查看帮助信息"
                    exit 1
                    ;;
            esac
        done
    fi
    
    # 执行构建
    local build_count=0
    
    if [ "$build_windows_flag" = true ]; then
        build_windows
        build_count=$((build_count + 1))
        echo ""
    fi
    
    if [ "$build_macos_intel_flag" = true ]; then
        build_macos_intel
        build_count=$((build_count + 1))
        echo ""
    fi
    
    if [ "$build_macos_arm_flag" = true ]; then
        build_macos_arm
        build_count=$((build_count + 1))
        echo ""
    fi
    
    if [ $build_count -eq 0 ]; then
        warn "没有选择要构建的平台"
        exit 1
    fi
    
    info "所有构建完成！"
    info ""
    info "构建产物位置:"
    if [ "$build_windows_flag" = true ]; then
        info "  Windows:   target/${WINDOWS_TARGET}/release/quickcap.exe"
    fi
    if [ "$build_macos_intel_flag" = true ]; then
        info "  macOS Intel: target/${MACOS_INTEL_TARGET}/release/quickcap"
    fi
    if [ "$build_macos_arm_flag" = true ]; then
        info "  macOS ARM:   target/${MACOS_ARM_TARGET}/release/quickcap"
    fi
}

# 运行主函数
main "$@"

