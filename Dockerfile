FROM ubuntu:18.04

RUN apt-get update && apt-get install -y curl python3
RUN apt-get install build-essential -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY ./ ./
RUN rustup default nightly
RUN cargo build --release
