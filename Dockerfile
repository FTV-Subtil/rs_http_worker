FROM rust:1.31-stretch as builder

ADD . ./

RUN apt update && \
    apt install -y libssl-dev && \
    cargo build --verbose --release && \
    cargo install

FROM debian:stretch
COPY --from=builder /usr/local/cargo/bin/http_worker /usr/bin

RUN apt update && apt install -y libssl1.1 ca-certificates

ENV AMQP_QUEUE job_http
ENV AMQP_COMPLETED_QUEUE job_http_completed
ENV AMQP_ERROR_QUEUE job_http_error
CMD http_worker
