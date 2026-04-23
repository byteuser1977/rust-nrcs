#!/bin/bash
set -e

echo "========================================="
echo "加密算法兼容性测试"
echo "========================================="

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 测试 Ed25519 签名兼容性
test_ed25519_compatibility() {
    log_info "测试 Ed25519 签名兼容性..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    # 运行 Rust 测试
    if cargo test -p crypto --test nrcs_compatibility -- --nocapture; then
        log_info "✓ Ed25519 兼容性测试通过"
        return 0
    else
        log_error "✗ Ed25519 兼容性测试失败"
        return 1
    fi
}

# 测试 NRCS 签名验证
test_nrcs_signature() {
    log_info "测试 NRCS 签名验证..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    if cargo test -p crypto --test nrcs_signature -- --nocapture; then
        log_info "✓ NRCS 签名验证测试通过"
        return 0
    else
        log_error "✗ NRCS 签名验证测试失败"
        return 1
    fi
}

# 测试 X25519 兼容性
test_x25519_compatibility() {
    log_info "测试 X25519 兼容性..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    if cargo test -p crypto --test x25519_compatibility -- --nocapture; then
        log_info "✓ X25519 兼容性测试通过"
        return 0
    else
        log_error "✗ X25519 兼容性测试失败"
        return 1
    fi
}

# 测试 SM2/SM3/SM4 国密算法
test_gm_algorithms() {
    log_info "测试 SM2/SM3/SM4 国密算法..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    # 运行所有 crypto 模块的测试
    if cargo test -p crypto --lib -- --nocapture; then
        log_info "✓ 国密算法测试通过"
        return 0
    else
        log_error "✗ 国密算法测试失败"
        return 1
    fi
}

# 主函数
main() {
    log_info "开始加密算法兼容性测试..."
    
    FAILED=0
    
    test_ed25519_compatibility || FAILED=$((FAILED + 1))
    test_nrcs_signature || FAILED=$((FAILED + 1))
    test_x25519_compatibility || FAILED=$((FAILED + 1))
    test_gm_algorithms || FAILED=$((FAILED + 1))
    
    log_info "========================================="
    if [ $FAILED -eq 0 ]; then
        log_info "所有加密算法测试通过！"
        exit 0
    else
        log_error "$FAILED 个测试失败"
        exit 1
    fi
}

main "$@"
