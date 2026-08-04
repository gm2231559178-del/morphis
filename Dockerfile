FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev gcc
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY auth-proxy/Cargo.toml auth-proxy/Cargo.toml
COPY identity-auth/Cargo.toml identity-auth/Cargo.toml
RUN mkdir -p src auth-proxy/src identity-auth/src && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'fn main() {}' > auth-proxy/src/main.rs && \
    echo '' > identity-auth/src/lib.rs && \
    cargo build --release -p morphis

COPY src/ src/
COPY identity-auth/src/ identity-auth/src/
RUN touch src/main.rs && cargo build --release -p morphis

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/morphis /usr/local/bin/morphis
WORKDIR /app
EXPOSE 4000
CMD ["morphis"]
