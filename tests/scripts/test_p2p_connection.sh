#!/bin/bash

# P2P连接测试脚本
# 每30秒尝试一次连接，直到成功或达到最大尝试次数

MAX_ATTEMPTS=30
ATTEMPT_INTERVAL=30

echo "开始P2P连接测试..."
echo "最大尝试次数: $MAX_ATTEMPTS"
echo "尝试间隔: $ATTEMPT_INTERVAL 秒"
echo "目标节点: 192.168.2.164:17974"
echo ""

for i in $(seq 1 $MAX_ATTEMPTS); do
    echo "========== 尝试 $i/$MAX_ATTEMPTS =========="
    echo "时间: $(date '+%Y-%m-%d %H:%M:%S')"
    
    # 启动Rust节点，运行30秒
    timeout 30 cargo run --release --bin nrcs-node 2>&1 | grep -E "(INFO|WARN|ERROR|blacklist|connected|handshake)" | head -20
    
    # 检查是否成功连接
    if echo "$output" | grep -q "Handshake successful"; then
        echo ""
        echo "✅ P2P连接成功！"
        exit 0
    fi
    
    # 如果还有尝试次数，等待一段时间
    if [ $i -lt $MAX_ATTEMPTS ]; then
        echo ""
        echo "等待 $ATTEMPT_INTERVAL 秒后重试..."
        sleep $ATTEMPT_INTERVAL
    fi
done

echo ""
echo "❌ 达到最大尝试次数，连接失败"
exit 1
