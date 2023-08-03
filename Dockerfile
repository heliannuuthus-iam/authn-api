# 第一层：构建阶段
FROM rust:latest as builder

# 设置工作目录
WORKDIR /app

COPY Cargo.lock Cargo.toml ./

RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

COPY ./src ./src

RUN cargo build --release

FROM alpine:latest

WORKDIR /app

# 从构建阶段复制构建好的二进制文件和静态文件到运行时镜像
COPY --from=builder /app/target/release/forum-server .

# 运行应用
CMD ["./forum-server"]