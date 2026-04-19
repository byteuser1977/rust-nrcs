#!/usr/bin/env bash
# 连接到 Java 节点的快捷脚本
# 用法: ./scripts/connect_java.sh [host] [port]

HOST="${1:-192.168.2.164}"
PORT="${2:-17974}"
API_PORT=17976

echo "🔗 尝试连接 P2P 节点 ${HOST}:${PORT} ..."

# 调用 Rust 节点的 connect RPC
curl -X POST "http://localhost:${API_PORT}/api/v1/peer/connect" \
  -H "Content-Type: application/json" \
  -d "{\"address\":\"${HOST}:${PORT}\"}" \
  -w "\n"

# 查询连接状态
echo -e "\n📊 当前对等节点列表:"
curl "http://localhost:${API_PORT}/api/v1/peer/peers"