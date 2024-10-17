FROM rust:1.81-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release && rm -rf src

COPY . .

FROM alpine:3.18

RUN apk add --no-cache ca-certificates openssl-dev

WORKDIR /usr/local/bin

COPY .env /usr/local/bin/

COPY --from=builder /usr/src/app/target/release/connect_x .

EXPOSE 80

CMD ["./connect_x"]