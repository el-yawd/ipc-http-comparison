FROM rust:1.87 as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin client

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/client /usr/local/bin/client

CMD ["client", "compare"]
