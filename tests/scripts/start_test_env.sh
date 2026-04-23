#!/bin/bash
set -e

echo "========================================="
echo "NRCS 集成测试环境启动脚本"
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
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-nrcs_test}"
DB_USER="${DB_USER:-nrcs}"
DB_PASSWORD="${DB_PASSWORD:-password}"

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

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo 未安装，请先安装 Rust"
        exit 1
    fi
    log_info "✓ Cargo 已安装"
    
    # 检查 PostgreSQL
    if ! command -v psql &> /dev/null; then
        log_error "PostgreSQL 客户端未安装"
        exit 1
    fi
    log_info "✓ PostgreSQL 客户端已安装"
    
    # 检查 jq
    if ! command -v jq &> /dev/null; then
        log_warn "jq 未安装，某些测试可能无法正常运行"
    else
        log_info "✓ jq 已安装"
    fi
    
    # 检查 curl
    if ! command -v curl &> /dev/null; then
        log_error "curl 未安装"
        exit 1
    fi
    log_info "✓ curl 已安装"
}

# 创建测试数据库
setup_database() {
    log_info "设置测试数据库..."
    
    # 检查数据库是否存在
    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
        log_warn "数据库 $DB_NAME 已存在，跳过创建"
    else
        log_info "创建数据库 $DB_NAME..."
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -c "CREATE DATABASE $DB_NAME;" || {
            log_error "创建数据库失败"
            exit 1
        }
        log_info "✓ 数据库创建成功"
    fi
    
    # 运行迁移
    log_info "运行数据库迁移..."
    cd /mnt/d/workspace/git/rust-nrcs
    cargo sqlx migrate run || {
        log_warn "数据库迁移失败，可能已经是最新的"
    }
}

# 编译 Rust 节点
build_rust_node() {
    log_info "编译 Rust NRCS 节点..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    cargo build --release --bin node || {
        log_error "编译 Rust 节点失败"
        exit 1
    }
    
    log_info "✓ Rust 节点编译成功"
}

# 创建测试配置
create_test_config() {
    log_info "创建测试配置..."
    
    cat > /mnt/d/workspace/git/rust-nrcs/config/local.toml <<EOF
# NRCS 集成测试配置

[p2p]
listen_addr = "/ip4/0.0.0.0/tcp/$RUST_PEER_PORT"
external_addr = "/ip4/0.0.0.0/tcp/$RUST_PEER_PORT"
bootstrap_nodes = [
  "/ip4/$JAVA_NRCS_HOST/tcp/$JAVA_PEER_PORT",
]
max_connections = 2000
connection_ttl_secs = 600
protocol_id = "/nrcs/1.0.0"

[api]
host = "0.0.0.0"
port = $RUST_NRCS_PORT
cors_allowed_origins = ["*"]
debug = true

[database]
url = "postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
pool_size = 10

[mining]
enabled = false

[consensus]
algorithm = "pos"
EOF
    
    log_info "✓ 测试配置创建成功"
}

# 检查 Java 节点连接
check_java_node() {
    log_info "检查 Java NRCS 节点连接..."
    
    if curl -s --connect-timeout 5 -X POST "http://$JAVA_NRCS_HOST:$JAVA_NRCS_PORT/nrcs" \
         -d "requestType=getBlockchainStatus" > /dev/null 2>&1; then
        log_info "✓ Java NRCS 节点可达"
    else
        log_warn "无法连接到 Java NRCS 节点 ($JAVA_NRCS_HOST:$JAVA_NRCS_PORT)"
        log_warn "某些测试可能无法执行"
    fi
}

# 启动 Rust 节点
start_rust_node() {
    log_info "启动 Rust NRCS 节点..."
    
    cd /mnt/d/workspace/git/rust-nrcs
    
    # 后台启动节点
    nohup ./target/release/node > tests/reports/rust_node.log 2>&1 &
    RUST_NODE_PID=$!
    
    echo $RUST_NODE_PID > tests/reports/rust_node.pid
    
    log_info "Rust 节点已启动 (PID: $RUST_NODE_PID)"
    
    # 等待节点启动
    log_info "等待节点启动..."
    for i in {1..30}; do
        if curl -s "http://$RUST_NRCS_HOST:$RUST_NRCS_PORT/health" > /dev/null 2>&1; then
            log_info "✓ Rust 节点已就绪"
            return 0
        fi
        sleep 1
    done
    
    log_error "Rust 节点启动超时"
    return 1
}

# 主函数
main() {
    log_info "开始设置集成测试环境..."
    
    check_dependencies
    setup_database
    build_rust_node
    create_test_config
    check_java_node
    start_rust_node
    
    log_info "========================================="
    log_info "集成测试环境设置完成！"
    log_info "========================================="
    log_info ""
    log_info "环境信息:"
    log_info "  - Rust NRCS API: http://$RUST_NRCS_HOST:$RUST_NRCS_PORT"
    log_info "  - Rust NRCS P2P: $RUST_PEER_PORT"
    log_info "  - Java NRCS API: http://$JAVA_NRCS_HOST:$JAVA_NRCS_PORT"
    log_info "  - Java NRCS P2P: $JAVA_PEER_PORT"
    log_info "  - 数据库: $DB_HOST:$DB_PORT/$DB_NAME"
    log_info ""
    log_info "日志文件: tests/reports/rust_node.log"
    log_info "PID 文件: tests/reports/rust_node.pid"
    log_info ""
    log_info "运行测试: ./tests/integration/run_all.sh"
    log_info "停止环境: ./tests/scripts/stop_test_env.sh"
}

main "$@"
