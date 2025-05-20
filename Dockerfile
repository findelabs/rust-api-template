FROM rust:1.85.1 as builder

RUN mkdir /app
RUN mkdir /app/bin

WORKDIR /app
COPY Cargo.toml .
COPY Cargo.lock .
COPY src ./src

RUN apt-get update && apt-get install -y libssl-dev pkg-config make
RUN cargo build --release
RUN cp ./target/release/main ./bin/main
RUN strip ./bin/main

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates openssl curl wget

COPY --from=builder /app/bin/ ./
COPY entrypoint.sh .

EXPOSE 8080
ENTRYPOINT ["/app/entrypoint.sh"]
