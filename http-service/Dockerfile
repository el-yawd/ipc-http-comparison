FROM rust:1.87 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin http-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/http-service /usr/local/bin/http-service

EXPOSE 3000
CMD ["http-service"]
