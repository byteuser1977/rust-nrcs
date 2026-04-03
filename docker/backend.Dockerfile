# 构建阶段 - Rust
FROM rust:1.75-slim AS builder

WORKDIR /app

# 安装构建依赖（用于编译原生依赖）
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 缓存依赖
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps

# 构建应用（生产构建）
RUN cargo build --release --bin nrcs-node --locked

# 运行阶段 - 精简镜像
FROM debian:bookworm-slim AS runtime

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/nrcs-node /usr/local/bin/
COPY --from=builder /app/config ./config

# 复制必要的资源文件
COPY --from=builder /app/crates/blockchain-types ./crates/blockchain-types
COPY --from=builder /app/crates/orm/migrations ./migrations

# 暴露端口
EXPOSE 8080 8081

# 创建非 root 用户
RUN useradd -m -u 1000 nrcs && chown -R nrcs:nrcs /app
USER nrcs

# 启动命令
CMD ["nrcs-node", "--config", "config/default.toml"]