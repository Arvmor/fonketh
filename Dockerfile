FROM rust:alpine AS builder

RUN apk update && apk add --no-cache musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release --bin app

FROM alpine:latest

COPY --from=builder /app/target/release/app /usr/local/bin/app

# SWARM UDP PORT
EXPOSE 7331/udp

CMD ["app"]