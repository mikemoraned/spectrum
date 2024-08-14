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
RUN ls -R .

CMD ./bin/api --opentelemetry --fgb-url https://spikes-remote-fgb.b-cdn.net/relations2.fgb
