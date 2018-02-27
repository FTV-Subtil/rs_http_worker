FROM rust:1.22

ADD . ./

WORKDIR rs_http_worker

RUN apt update && \
    apt install -y libssl-dev && \
    cargo build --verbose --release && \
    cargo install

ENV PATH "$PATH:/root/.cargo/bin/"

CMD http_worker

