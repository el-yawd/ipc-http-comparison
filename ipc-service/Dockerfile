FROM rust:1.87 as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin ipc-service

FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/target/release/ipc-service /usr/local/bin/ipc-service

CMD ["ipc-service"]
