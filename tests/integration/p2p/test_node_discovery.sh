#!/bin/bash
set -e

echo "========================================="
echo "P2P 网络互通测试 - 节点发现"
echo "========================================="

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 配置变量
JAVA_NRCS_HOST="${JAVA_NRCS_HOST:-192.168.2.164}"
JAVA_NRCS_PORT="${JAVA_NRCS_PORT:-17976}"
JAVA_PEER_PORT="${JAVA_PEER_PORT:-17974}"
RUST_NRCS_HOST="${RUST_NRCS_HOST:-localhost}"
RUST_NRCS_PORT="${RUST_NRCS_PORT:-17976}"
RUST_PEER_PORT="${RUST_PEER_PORT:-17974}"

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

# 测试 Java 节点连接
test_java_node_connection() {
    log_info "测试 Java NRCS 节点连接..."
    
    if curl -s --connect-timeout 5 -X POST "http://$JAVA_NRCS_HOST:$JAVA_NRCS_PORT/nrcs" \
         -d "requestType=getBlockchainStatus" > /tmp/java_status.json 2>&1; then
        log_info "✓ Java NRCS 节点可达"
        
        # 显示节点状态
        log_info "Java 节点状态:"
        cat /tmp/java_status.json | jq '.' || cat /tmp/java_status.json
        
        return 0
    else
        log_error "✗ 无法连接到 Java NRCS 节点"
        return 1
    fi
}

# 测试 Rust 节点连接
test_rust_node_connection() {
    log_info "测试 Rust NRCS 节点连接..."
    
    if curl -s --connect-timeout 5 "http://$RUST_NRCS_HOST:$RUST_NRCS_PORT/health" > /tmp/rust_health.json 2>&1; then
        log_info "✓ Rust NRCS 节点可达"
        
        # 显示节点状态
        log_info "Rust 节点状态:"
        cat /tmp/rust_health.json | jq '.' || cat /tmp/rust_health.json
        
        return 0
    else
        log_error "✗ 无法连接到 Rust NRCS 节点"
        return 1
    fi
}

# 测试 getInfo 交换
test_getinfo_exchange() {
    log_info "测试 getInfo 交换..."
    
    # 从 Rust 节点获取信息
    log_info "从 Rust 节点获取 getInfo..."
    RUST_INFO=$(curl -s -X POST "http://$RUST_NRCS_HOST:$RUST_PEER_PORT/peer" \
        -H "Content-Type: application/json" \
        -d '{"requestType":"getInfo","protocol":1}')
    
    if [ $? -eq 0 ]; then
        log_info "Rust 节点 getInfo 响应:"
        echo "$RUST_INFO" | jq '.' || echo "$RUST_INFO"
    else
        log_error "✗ Rust 节点 getInfo 失败"
        return 1
    fi
    
    # 从 Java 节点获取信息
    log_info "从 Java 节点获取 getInfo..."
    JAVA_INFO=$(curl -s -X POST "http://$JAVA_NRCS_HOST:$JAVA_PEER_PORT/peer" \
        -H "Content-Type: application/json" \
        -d '{"requestType":"getInfo","protocol":1}')
    
    if [ $? -eq 0 ]; then
        log_info "Java 节点 getInfo 响应:"
        echo "$JAVA_INFO" | jq '.' || echo "$JAVA_INFO"
    else
        log_error "✗ Java 节点 getInfo 失败"
        return 1
    fi
    
    # 对比响应字段
    log_info "对比响应字段..."
    
    RUST_FIELDS=$(echo "$RUST_INFO" | jq 'keys | sort' 2>/dev/null || echo "[]")
    JAVA_FIELDS=$(echo "$JAVA_INFO" | jq 'keys | sort' 2>/dev/null || echo "[]")
    
    if [ "$RUST_FIELDS" == "$JAVA_FIELDS" ]; then
        log_info "✓ getInfo 响应字段一致"
        return 0
    else
        log_warn "getInfo 响应字段不一致"
        log_info "Rust 字段: $RUST_FIELDS"
        log_info "Java 字段: $JAVA_FIELDS"
        return 1
    fi
}

# 测试节点互相发现
test_peer_discovery() {
    log_info "测试节点互相发现..."
    
    # 获取 Rust 节点的 peers 列表
    log_info "获取 Rust 节点的 peers 列表..."
    RUST_PEERS=$(curl -s -X POST "http://$RUST_NRCS_HOST:$RUST_PEER_PORT/peer" \
        -H "Content-Type: application/json" \
        -d '{"requestType":"getPeers","protocol":1}')
    
    log_info "Rust 节点 peers:"
    echo "$RUST_PEERS" | jq '.' || echo "$RUST_PEERS"
    
    # 获取 Java 节点的 peers 列表
    log_info "获取 Java 节点的 peers 列表..."
    JAVA_PEERS=$(curl -s -X POST "http://$JAVA_NRCS_HOST:$JAVA_PEER_PORT/peer" \
        -H "Content-Type: application/json" \
        -d '{"requestType":"getPeers","protocol":1}')
    
    log_info "Java 节点 peers:"
    echo "$JAVA_PEERS" | jq '.' || echo "$JAVA_PEERS"
    
    # 检查是否互相发现
    RUST_PEER_COUNT=$(echo "$RUST_PEERS" | jq '.peers | length' 2>/dev/null || echo "0")
    JAVA_PEER_COUNT=$(echo "$JAVA_PEERS" | jq '.peers | length' 2>/dev/null || echo "0")
    
    log_info "Rust 节点发现 $RUST_PEER_COUNT 个 peers"
    log_info "Java 节点发现 $JAVA_PEER_COUNT 个 peers"
    
    if [ "$RUST_PEER_COUNT" -gt 0 ] && [ "$JAVA_PEER_COUNT" -gt 0 ]; then
        log_info "✓ 节点互相发现成功"
        return 0
    else
        log_warn "节点互相发现失败，可能需要等待更长时间"
        return 1
    fi
}

# 主函数
main() {
    log_info "开始 P2P 网络互通测试..."
    
    FAILED=0
    
    test_java_node_connection || FAILED=$((FAILED + 1))
    test_rust_node_connection || FAILED=$((FAILED + 1))
    test_getinfo_exchange || FAILED=$((FAILED + 1))
    test_peer_discovery || FAILED=$((FAILED + 1))
    
    log_info "========================================="
    if [ $FAILED -eq 0 ]; then
        log_info "所有 P2P 网络测试通过！"
        exit 0
    else
        log_error "$FAILED 个测试失败"
        exit 1
    fi
}

main "$@"
