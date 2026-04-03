# 构建阶段
FROM node:18-alpine AS builder

WORKDIR /app

# 复制 package 文件
COPY package.json package-lock.json ./
COPY frontend/package.json ./frontend/package.json

# 安装依赖
RUN npm ci --only=production

# 复制源代码
COPY frontend/ .

# 构建生产版本
RUN npm run build

# 生产运行阶段 - Nginx
FROM nginx:alpine AS runtime

# 复制自定义 Nginx 配置
COPY nginx.conf /etc/nginx/nginx.conf

# 从构建阶段复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html

# 复制环境变量配置脚本
COPY frontend/env.sh /docker-entrypoint.d/env.sh
RUN chmod +x /docker-entrypoint.d/env.sh

# 暴露端口
EXPOSE 80

# 使用入口脚本动态注入环境变量
ENTRYPOINT ["/docker-entrypoint.d/env.sh"]
CMD ["nginx", "-g", "daemon off;"]