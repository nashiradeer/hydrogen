FROM rust:1.75.0-alpine3.19 AS builder
WORKDIR /app
RUN apk -U upgrade
RUN apk add libc-dev
COPY assets/ assets/
COPY src/ src/
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build -r
RUN rm assets/langs/en-US.json

FROM alpine:3.19.0
WORKDIR /app
RUN apk -U upgrade
COPY --from=builder /app/target/release/hydrogen hydrogen
COPY --from=builder /app/assets/langs lang/
ENV RUST_LOG=hydrogen=info
ENV LANGUAGE_PATH=/app/lang
CMD ["/app/hydrogen"]