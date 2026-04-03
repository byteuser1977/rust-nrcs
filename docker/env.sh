#!/bin/sh
# 动态设置前端环境变量，从容器环境或 .env 文件读取

set -e

ENV_FILE="/app/.env.docker"
OUTPUT_FILE="/usr/share/nginx/html/env.js"

echo "Setting up frontend environment variables..."

# 默认值
VITE_API_BASE_URL="${VITE_API_BASE_URL:-http://localhost:8080}"
VITE_WS_URL="${VITE_WS_URL:-ws://localhost:8081}"
VITE_APP_TITLE="${VITE_APP_TITLE:-NRCS Blockchain}"
VITE_APP_VERSION="${VITE_APP_VERSION:-1.0.0}"

# 如果存在配置文件，则覆蓋
cat > ${OUTPUT_FILE} << EOF
window._env_ = {
  VITE_API_BASE_URL: "${VITE_API_BASE_URL}",
  VITE_WS_URL: "${VITE_WS_URL}",
  VITE_APP_TITLE: "${VITE_APP_TITLE}",
  VITE_APP_VERSION: "${VITE_APP_VERSION}"
};
EOF

echo "Environment variables written to ${OUTPUT_FILE}"
echo "API URL: ${VITE_API_BASE_URL}"
echo "WS URL: ${VITE_WS_URL}"

# 启动 Nginx
exec nginx -g 'daemon off;'