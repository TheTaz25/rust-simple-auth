FROM rust:1.77-alpine3.18 as builder

RUN apk add --no-cache libpq mysql-client pkgconfig openssl-dev libc-dev

WORKDIR /usr/src/rust-auth
COPY . .

RUN cargo build -r

RUN ls -la

# ---

FROM alpine:3.18

COPY --from=builder /usr/src/rust-auth/target/release/rust-auth /usr/local/bin/rust-auth

EXPOSE 8080
CMD ["rust-auth"]
