FROM rust:latest

WORKDIR /app

COPY target/release/forum-api .

CMD ["./forum-api"]