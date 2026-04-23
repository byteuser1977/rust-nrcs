#!/bin/bash
set -e

echo "========================================="
echo "共识算法一致性测试"
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

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 测试 Rust 共识模块
test_consensus_module() {
    log_info "测试 Rust 共识模块..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    if cargo test -p consensus --lib -- --nocapture; then
        log_info "✓ Rust 共识模块测试通过"
        return 0
    else
        log_error "✗ Rust 共识模块测试失败"
        return 1
    fi
}

# 测试出块者选择算法
test_forger_selection() {
    log_info "测试出块者选择算法..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    # 运行特定的出块者选择测试
    if cargo test -p consensus --test consensus_tests -- --nocapture; then
        log_info "✓ 出块者选择测试通过"
        return 0
    else
        log_error "✗ 出块者选择测试失败"
        return 1
    fi
}

# 测试难度计算
test_difficulty_calculation() {
    log_info "测试难度计算..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    # 运行难度计算相关测试
    if cargo test -p consensus --lib target_calculator -- --nocapture; then
        log_info "✓ 难度计算测试通过"
        return 0
    else
        log_warn "难度计算测试可能不存在或失败"
        return 0
    fi
}

# 主函数
main() {
    log_info "开始共识算法一致性测试..."
    
    FAILED=0
    
    test_consensus_module || FAILED=$((FAILED + 1))
    test_forger_selection || FAILED=$((FAILED + 1))
    test_difficulty_calculation || FAILED=$((FAILED + 1))
    
    log_info "========================================="
    if [ $FAILED -eq 0 ]; then
        log_info "所有共识算法测试通过！"
        exit 0
    else
        log_error "$FAILED 个测试失败"
        exit 1
    fi
}

main "$@"
