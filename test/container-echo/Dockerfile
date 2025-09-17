FROM rust:alpine AS builder
COPY ./ /app
WORKDIR /app
RUN apk add musl-dev
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/container-echo /container-echo
ENTRYPOINT ["/container-echo"]
EXPOSE 8080
