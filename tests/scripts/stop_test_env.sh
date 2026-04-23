#!/bin/bash
set -e

echo "========================================="
echo "NRCS 集成测试环境停止脚本"
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

# 停止 Rust 节点
stop_rust_node() {
    log_info "停止 Rust NRCS 节点..."
    
    PID_FILE="/mnt/d/workspace/git/rust-nrcs/tests/reports/rust_node.pid"
    
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        
        if kill -0 "$PID" 2>/dev/null; then
            log_info "发送 SIGTERM 到进程 $PID..."
            kill "$PID"
            
            # 等待进程结束
            for i in {1..10}; do
                if ! kill -0 "$PID" 2>/dev/null; then
                    log_info "✓ Rust 节点已停止"
                    rm -f "$PID_FILE"
                    return 0
                fi
                sleep 1
            done
            
            # 如果进程还在运行，强制杀死
            log_warn "进程未响应，发送 SIGKILL..."
            kill -9 "$PID" 2>/dev/null || true
            rm -f "$PID_FILE"
            log_info "✓ Rust 节点已强制停止"
        else
            log_warn "进程 $PID 不存在"
            rm -f "$PID_FILE"
        fi
    else
        log_warn "PID 文件不存在，Rust 节点可能未运行"
    fi
}

# 清理临时文件
cleanup() {
    log_info "清理临时文件..."
    
    # 清理日志文件（可选）
    # rm -f /mnt/d/workspace/git/rust-nrcs/tests/reports/*.log
    
    log_info "✓ 临时文件清理完成"
}

# 主函数
main() {
    stop_rust_node
    cleanup
    
    log_info "========================================="
    log_info "集成测试环境已停止"
    log_info "========================================="
}

main "$@"
