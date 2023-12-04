FROM rust:1.74.0-slim-buster as builder

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:1.74.0-slim-buster,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/fly ./fly

# Runtime image
# FROM debian:buster-slim
# FROM debian:buster
FROM rust:1.74.0-slim-buster

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/fly /app/fly

# Run the app
CMD ./fly