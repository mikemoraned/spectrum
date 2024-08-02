FROM rust:slim-buster AS builder

WORKDIR /prod

# following needed for `openssl-sys v0.9.98`:
RUN apt-get update -y
RUN apt-get install -y pkg-config
RUN apt-get install -y libssl-dev

COPY . .
RUN cargo build --release

FROM fedora:34 AS runner
COPY --from=builder /prod/target/release/api /bin
COPY --from=builder /prod/api/data /data
RUN ls -R .

CMD ./bin/api --opentelemetry --fgb ./data/all.fgb
