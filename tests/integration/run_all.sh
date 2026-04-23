#!/bin/bash
set -e

echo "========================================="
echo "NRCS 集成测试 - 运行所有测试"
echo "========================================="

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置变量
TEST_DIR="/mnt/d/workspace/git/rust-nrcs/tests/integration"
REPORT_DIR="/mnt/d/workspace/git/rust-nrcs/tests/reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="$REPORT_DIR/test_report_$TIMESTAMP.md"

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

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

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

# 运行测试套件
run_test_suite() {
    local suite_name=$1
    local suite_dir=$2
    
    log_info "运行测试套件: $suite_name"
    
    if [ -d "$suite_dir" ]; then
        for test_script in "$suite_dir"/*.sh; do
            if [ -f "$test_script" ]; then
                test_name=$(basename "$test_script" .sh)
                log_test "执行测试: $test_name"
                
                TOTAL_TESTS=$((TOTAL_TESTS + 1))
                
                if bash "$test_script" >> "$REPORT_DIR/test_output_$TIMESTAMP.log" 2>&1; then
                    log_info "✓ $test_name 通过"
                    PASSED_TESTS=$((PASSED_TESTS + 1))
                    echo "- [✓] $test_name" >> "$REPORT_FILE"
                else
                    log_error "✗ $test_name 失败"
                    FAILED_TESTS=$((FAILED_TESTS + 1))
                    echo "- [✗] $test_name" >> "$REPORT_FILE"
                fi
            fi
        done
    else
        log_warn "测试套件目录不存在: $suite_dir"
    fi
}

# 运行 Rust 单元测试
run_rust_tests() {
    log_info "运行 Rust 单元测试..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if cargo test --lib --all >> "$REPORT_DIR/test_output_$TIMESTAMP.log" 2>&1; then
        log_info "✓ Rust 单元测试通过"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "- [✓] Rust 单元测试" >> "$REPORT_FILE"
    else
        log_error "✗ Rust 单元测试失败"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "- [✗] Rust 单元测试" >> "$REPORT_FILE"
    fi
}

# 生成测试报告
generate_report() {
    log_info "生成测试报告..."
    
    cat > "$REPORT_FILE" <<EOF
# NRCS 集成测试报告

**测试时间**: $(date +"%Y-%m-%d %H:%M:%S")  
**测试环境**: $(uname -a)

## 测试概要

| 指标 | 数值 |
|------|------|
| 总测试数 | $TOTAL_TESTS |
| 通过数 | $PASSED_TESTS |
| 失败数 | $FAILED_TESTS |
| 通过率 | $(awk "BEGIN {printf \"%.2f%%\", ($PASSED_TESTS/$TOTAL_TESTS)*100}") |

## 测试结果详情

EOF

    # 添加各测试套件结果
    echo "### 加密算法兼容性测试" >> "$REPORT_FILE"
    run_test_suite "加密算法" "$TEST_DIR/../crypto" || true
    
    echo "" >> "$REPORT_FILE"
    echo "### P2P 网络互通测试" >> "$REPORT_FILE"
    run_test_suite "P2P 网络" "$TEST_DIR/p2p" || true
    
    echo "" >> "$REPORT_FILE"
    echo "### HTTP API 兼容性测试" >> "$REPORT_FILE"
    run_test_suite "HTTP API" "$TEST_DIR/api" || true
    
    echo "" >> "$REPORT_FILE"
    echo "### 共识算法一致性测试" >> "$REPORT_FILE"
    run_test_suite "共识算法" "$TEST_DIR/consensus" || true
    
    # 添加 Rust 单元测试结果
    echo "" >> "$REPORT_FILE"
    echo "### Rust 单元测试" >> "$REPORT_FILE"
    run_rust_tests
    
    # 添加结论
    cat >> "$REPORT_FILE" <<EOF

## 测试结论

EOF

    if [ $FAILED_TESTS -eq 0 ]; then
        echo "✅ **所有测试通过！**" >> "$REPORT_FILE"
    else
        echo "⚠️ **部分测试失败，请检查详细日志。**" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "失败测试数: $FAILED_TESTS" >> "$REPORT_FILE"
    fi
    
    echo "" >> "$REPORT_FILE"
    echo "---" >> "$REPORT_FILE"
    echo "**详细日志**: test_output_$TIMESTAMP.log" >> "$REPORT_FILE"
    
    log_info "测试报告已生成: $REPORT_FILE"
}

# 主函数
main() {
    log_info "开始运行集成测试..."
    
    # 创建报告目录
    mkdir -p "$REPORT_DIR"
    
    # 运行测试
    generate_report
    
    # 显示结果
    log_info "========================================="
    log_info "测试执行完成"
    log_info "========================================="
    log_info "总测试数: $TOTAL_TESTS"
    log_info "通过数: $PASSED_TESTS"
    log_info "失败数: $FAILED_TESTS"
    log_info "通过率: $(awk "BEGIN {printf \"%.2f%%\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")"
    log_info ""
    log_info "测试报告: $REPORT_FILE"
    log_info "详细日志: $REPORT_DIR/test_output_$TIMESTAMP.log"
    
    # 返回退出码
    if [ $FAILED_TESTS -eq 0 ]; then
        exit 0
    else
        exit 1
    fi
}

main "$@"
