FROM rust:1.36-stretch as builder

ADD . ./

RUN apt-get update && \
    apt-get install -y libssl-dev && \
    cargo build -j1 --verbose --release && \
    cargo install

FROM debian:stretch
COPY --from=builder /usr/local/cargo/bin/http_worker /usr/bin

RUN apt update && apt install -y libssl1.1 ca-certificates

ENV AMQP_QUEUE job_http
ENV AMQP_COMPLETED_QUEUE job_http_completed
ENV AMQP_ERROR_QUEUE job_http_error
CMD http_worker
