FROM rust:1-alpine3.18 AS builder
WORKDIR /app
RUN apk -U upgrade
RUN apk add libc-dev
COPY src/ src/
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build -r

FROM alpine:3.18
WORKDIR /app
COPY --from=builder /app/target/release/hydrogen /app/hydrogen
COPY assets/langs/ lang/
ENV LANGUAGE_PATH=/app/lang
CMD ["/app/hydrogen"]