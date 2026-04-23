#!/bin/bash
set -e

echo "========================================="
echo "HTTP API 兼容性测试"
echo "========================================="

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 配置变量
JAVA_NRCS_HOST="${JAVA_NRCS_HOST:-192.168.2.164}"
JAVA_NRCS_PORT="${JAVA_NRCS_PORT:-17976}"
RUST_NRCS_HOST="${RUST_NRCS_HOST:-localhost}"
RUST_NRCS_PORT="${RUST_NRCS_PORT:-17976}"

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

# 测试 API 端点
test_api_endpoint() {
    local endpoint=$1
    local description=$2
    
    log_info "测试 API: $description ($endpoint)"
    
    # 获取 Java 响应
    JAVA_RESP=$(curl -s -X POST "http://$JAVA_NRCS_HOST:$JAVA_NRCS_PORT/nrcs?requestType=$endpoint")
    JAVA_STATUS=$?
    
    # 获取 Rust 响应
    RUST_RESP=$(curl -s -X POST "http://$RUST_NRCS_HOST:$RUST_NRCS_PORT/nrcs?requestType=$endpoint")
    RUST_STATUS=$?
    
    # 检查响应状态
    if [ $JAVA_STATUS -ne 0 ]; then
        log_warn "Java 节点请求失败: $endpoint"
        JAVA_RESP="{}"
    fi
    
    if [ $RUST_STATUS -ne 0 ]; then
        log_error "Rust 节点请求失败: $endpoint"
        return 1
    fi
    
    # 对比响应字段
    JAVA_FIELDS=$(echo "$JAVA_RESP" | jq 'keys | sort' 2>/dev/null || echo "[]")
    RUST_FIELDS=$(echo "$RUST_RESP" | jq 'keys | sort' 2>/dev/null || echo "[]")
    
    if [ "$JAVA_FIELDS" == "$RUST_FIELDS" ]; then
        log_info "✓ $description 字段一致"
        return 0
    else
        log_warn "$description 字段不一致"
        log_info "  Java: $JAVA_FIELDS"
        log_info "  Rust: $RUST_FIELDS"
        
        # 检查是否有错误字段
        if echo "$RUST_RESP" | jq -e '.error' > /dev/null 2>&1; then
            log_error "Rust 返回错误: $(echo "$RUST_RESP" | jq -r '.error')"
            return 1
        fi
        
        return 1
    fi
}

# 测试基础 API 端点
test_basic_apis() {
    log_info "测试基础 API 端点..."
    
    FAILED=0
    
    # 基础 API 列表
    declare -A BASIC_APIS=(
        ["getBlockchainStatus"]="获取区块链状态"
        ["getTime"]="获取时间"
        ["getConstants"]="获取常量"
        ["getPeers"]="获取节点列表"
    )
    
    for endpoint in "${!BASIC_APIS[@]}"; do
        description="${BASIC_APIS[$endpoint]}"
        test_api_endpoint "$endpoint" "$description" || FAILED=$((FAILED + 1))
    done
    
    return $FAILED
}

# 测试账户相关 API
test_account_apis() {
    log_info "测试账户相关 API..."
    
    FAILED=0
    
    # 使用测试账户 ID
    TEST_ACCOUNT="123456789"
    
    # 账户 API 列表
    declare -A ACCOUNT_APIS=(
        ["getAccount&account=$TEST_ACCOUNT"]="获取账户信息"
        ["getBalance&account=$TEST_ACCOUNT"]="获取账户余额"
        ["getAccountPublicKey&account=$TEST_ACCOUNT"]="获取账户公钥"
    )
    
    for endpoint in "${!ACCOUNT_APIS[@]}"; do
        description="${ACCOUNT_APIS[$endpoint]}"
        test_api_endpoint "$endpoint" "$description" || FAILED=$((FAILED + 1))
    done
    
    return $FAILED
}

# 测试区块相关 API
test_block_apis() {
    log_info "测试区块相关 API..."
    
    FAILED=0
    
    # 区块 API 列表
    declare -A BLOCK_APIS=(
        ["getBlock&height=1"]="获取区块（按高度）"
        ["getBlock&block=1"]="获取区块（按ID）"
        ["getBlocks&firstIndex=0&lastIndex=10"]="获取区块列表"
    )
    
    for endpoint in "${!BLOCK_APIS[@]}"; do
        description="${BLOCK_APIS[$endpoint]}"
        test_api_endpoint "$endpoint" "$description" || FAILED=$((FAILED + 1))
    done
    
    return $FAILED
}

# 测试交易相关 API
test_transaction_apis() {
    log_info "测试交易相关 API..."
    
    FAILED=0
    
    # 交易 API 列表
    declare -A TRANSACTION_APIS=(
        ["getUnconfirmedTransactions"]="获取未确认交易"
        ["getExpectedTransactions"]="获取预期交易"
    )
    
    for endpoint in "${!TRANSACTION_APIS[@]}"; do
        description="${TRANSACTION_APIS[$endpoint]}"
        test_api_endpoint "$endpoint" "$description" || FAILED=$((FAILED + 1))
    done
    
    return $FAILED
}

# 测试工具 API
test_utility_apis() {
    log_info "测试工具 API..."
    
    FAILED=0
    
    # 工具 API 列表
    declare -A UTILITY_APIS=(
        ["hash&secret=test&secretIsText=true"]="哈希计算"
        ["hexConvert&string=hello"]="十六进制转换"
        ["longConvert&id=123456789"]="长整型转换"
        ["rsConvert&account=123456789"]="RS 地址转换"
    )
    
    for endpoint in "${!UTILITY_APIS[@]}"; do
        description="${UTILITY_APIS[$endpoint]}"
        test_api_endpoint "$endpoint" "$description" || FAILED=$((FAILED + 1))
    done
    
    return $FAILED
}

# 测试 POST API
test_post_apis() {
    log_info "测试 POST API..."
    
    # 测试 sendMoney API
    log_info "测试 sendMoney API..."
    
    SEND_MONEY_DATA="requestType=sendMoney&recipient=987654321&amountNQT=1000000000&feeNQT=100000000&deadline=1440&secretPhrase=test_secret"
    
    RUST_RESP=$(curl -s -X POST "http://$RUST_NRCS_HOST:$RUST_NRCS_PORT/nrcs" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "$SEND_MONEY_DATA")
    
    # 检查响应
    if echo "$RUST_RESP" | jq -e '.transaction' > /dev/null 2>&1; then
        log_info "✓ sendMoney API 响应正常"
        return 0
    elif echo "$RUST_RESP" | jq -e '.error' > /dev/null 2>&1; then
        # 预期的错误（如密钥无效）
        log_info "✓ sendMoney API 返回预期错误: $(echo "$RUST_RESP" | jq -r '.error')"
        return 0
    else
        log_error "✗ sendMoney API 响应异常"
        log_info "响应: $RUST_RESP"
        return 1
    fi
}

# 主函数
main() {
    log_info "开始 HTTP API 兼容性测试..."
    
    TOTAL_FAILED=0
    
    test_basic_apis || TOTAL_FAILED=$((TOTAL_FAILED + $?))
    test_account_apis || TOTAL_FAILED=$((TOTAL_FAILED + $?))
    test_block_apis || TOTAL_FAILED=$((TOTAL_FAILED + $?))
    test_transaction_apis || TOTAL_FAILED=$((TOTAL_FAILED + $?))
    test_utility_apis || TOTAL_FAILED=$((TOTAL_FAILED + $?))
    test_post_apis || TOTAL_FAILED=$((TOTAL_FAILED + 1))
    
    log_info "========================================="
    if [ $TOTAL_FAILED -eq 0 ]; then
        log_info "所有 HTTP API 测试通过！"
        exit 0
    else
        log_error "$TOTAL_FAILED 个 API 测试失败"
        exit 1
    fi
}

main "$@"
