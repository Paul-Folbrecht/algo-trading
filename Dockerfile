FROM rust:1.80.1 AS builder
COPY . .
RUN cargo build

FROM debian:bookworm-slim AS runner
COPY --from=builder /target/ ./target/
COPY --from=builder /app_config/ ./app_config/
COPY --from=builder /config/ ./config/
RUN apt-get update && apt install -y openssl
RUN apt-get install ca-certificates -y
CMD ["/target/release/server"]
