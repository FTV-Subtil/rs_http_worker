FROM rust:1.24-stretch as builder

ADD . ./

RUN apt update && \
    apt install -y libssl-dev && \
    cargo build --verbose --release && \
    cargo install

FROM debian:stretch
COPY --from=builder /usr/local/cargo/bin/http_worker /usr/bin

RUN apt update && apt install -y libssl1.1 ca-certificates

CMD http_worker
