#!/bin/bash
set -e

echo "========================================="
echo "P2P 网络互通测试 - 区块同步"
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

# 获取区块高度
get_block_height() {
    local host=$1
    local port=$2
    
    HEIGHT=$(curl -s -X POST "http://$host:$port/nrcs" \
        -d "requestType=getBlockchainStatus" | jq -r '.numberOfBlocks' 2>/dev/null || echo "0")
    
    echo "$HEIGHT"
}

# 获取区块哈希
get_block_hash() {
    local host=$1
    local port=$2
    local height=$3
    
    HASH=$(curl -s -X POST "http://$host:$port/nrcs" \
        -d "requestType=getBlock&height=$height" | jq -r '.blockSignature' 2>/dev/null || echo "")
    
    echo "$HASH"
}

# 测试区块高度同步
test_block_height_sync() {
    log_info "测试区块高度同步..."
    
    # 获取 Java 节点区块高度
    JAVA_HEIGHT=$(get_block_height "$JAVA_NRCS_HOST" "$JAVA_NRCS_PORT")
    log_info "Java 节点区块高度: $JAVA_HEIGHT"
    
    # 获取 Rust 节点区块高度
    RUST_HEIGHT=$(get_block_height "$RUST_NRCS_HOST" "$RUST_NRCS_PORT")
    log_info "Rust 节点区块高度: $RUST_HEIGHT"
    
    if [ "$JAVA_HEIGHT" -eq 0 ]; then
        log_warn "Java 节点没有区块，跳过同步测试"
        return 0
    fi
    
    # 等待同步（最多 60 秒）
    log_info "等待区块同步..."
    for i in {1..60}; do
        RUST_HEIGHT=$(get_block_height "$RUST_NRCS_HOST" "$RUST_NRCS_PORT")
        
        if [ "$RUST_HEIGHT" -ge "$JAVA_HEIGHT" ]; then
            log_info "✓ 区块高度同步完成"
            return 0
        fi
        
        log_info "同步进度: $RUST_HEIGHT / $JAVA_HEIGHT"
        sleep 1
    done
    
    log_error "✗ 区块同步超时"
    return 1
}

# 测试区块数据一致性
test_block_data_consistency() {
    log_info "测试区块数据一致性..."
    
    # 获取 Java 节点区块高度
    JAVA_HEIGHT=$(get_block_height "$JAVA_NRCS_HOST" "$JAVA_NRCS_PORT")
    
    if [ "$JAVA_HEIGHT" -eq 0 ]; then
        log_warn "Java 节点没有区块，跳过一致性测试"
        return 0
    fi
    
    # 测试多个区块高度
    TEST_HEIGHTS=(1 10 50 100)
    
    if [ "$JAVA_HEIGHT" -lt 100 ]; then
        # 如果区块高度不足，测试现有区块
        TEST_HEIGHTS=($(seq 1 $JAVA_HEIGHT))
    fi
    
    FAILED_COUNT=0
    
    for HEIGHT in "${TEST_HEIGHTS[@]}"; do
        if [ "$HEIGHT" -le "$JAVA_HEIGHT" ]; then
            log_info "测试区块高度 $HEIGHT..."
            
            JAVA_HASH=$(get_block_hash "$JAVA_NRCS_HOST" "$JAVA_NRCS_PORT" "$HEIGHT")
            RUST_HASH=$(get_block_hash "$RUST_NRCS_HOST" "$RUST_NRCS_PORT" "$HEIGHT")
            
            if [ "$JAVA_HASH" == "$RUST_HASH" ] && [ -n "$JAVA_HASH" ]; then
                log_info "✓ 区块 $HEIGHT 哈希一致"
            else
                log_error "✗ 区块 $HEIGHT 哈希不一致"
                log_info "  Java: $JAVA_HASH"
                log_info "  Rust: $RUST_HASH"
                FAILED_COUNT=$((FAILED_COUNT + 1))
            fi
        fi
    done
    
    if [ $FAILED_COUNT -eq 0 ]; then
        log_info "✓ 所有测试区块数据一致"
        return 0
    else
        log_error "✗ $FAILED_COUNT 个区块数据不一致"
        return 1
    fi
}

# 测试区块字段完整性
test_block_fields() {
    log_info "测试区块字段完整性..."
    
    # 获取一个区块
    BLOCK=$(curl -s -X POST "http://$RUST_NRCS_HOST:$RUST_NRCS_PORT/nrcs" \
        -d "requestType=getBlock&height=1")
    
    # 检查必需字段
    REQUIRED_FIELDS=("version" "timestamp" "previousBlock" "totalAmountNQT" "totalFeeNQT" \
                     "payloadLength" "payloadHash" "generatorPublicKey" "generationSignature" \
                     "blockSignature" "transactions")
    
    MISSING_FIELDS=0
    
    for FIELD in "${REQUIRED_FIELDS[@]}"; do
        if echo "$BLOCK" | jq -e ".$FIELD" > /dev/null 2>&1; then
            log_info "✓ 字段 $FIELD 存在"
        else
            log_error "✗ 字段 $FIELD 缺失"
            MISSING_FIELDS=$((MISSING_FIELDS + 1))
        fi
    done
    
    if [ $MISSING_FIELDS -eq 0 ]; then
        log_info "✓ 所有必需字段存在"
        return 0
    else
        log_error "✗ $MISSING_FIELDS 个字段缺失"
        return 1
    fi
}

# 主函数
main() {
    log_info "开始区块同步测试..."
    
    FAILED=0
    
    test_block_height_sync || FAILED=$((FAILED + 1))
    test_block_data_consistency || FAILED=$((FAILED + 1))
    test_block_fields || FAILED=$((FAILED + 1))
    
    log_info "========================================="
    if [ $FAILED -eq 0 ]; then
        log_info "所有区块同步测试通过！"
        exit 0
    else
        log_error "$FAILED 个测试失败"
        exit 1
    fi
}

main "$@"
