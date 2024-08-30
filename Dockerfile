FROM rust:1.80.1 AS builder
COPY . .
RUN cargo build --release #--target x86_64-unknown-linux-gnu

FROM debian:bookworm-slim AS runner
COPY --from=builder /target/ ./target/
COPY --from=builder /app_config/ ./app_config/
COPY --from=builder /config/ ./config/
RUN apt-get update && apt install -y openssl
#RUN apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
RUN apt-get install ca-certificates -y
CMD ["/target/release/server"]
