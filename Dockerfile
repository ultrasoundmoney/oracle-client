# syntax=docker/dockerfile:1
FROM golang:1.20-alpine

RUN apk add git

RUN git clone https://github.com/chronicleprotocol/oracle-suite.git
WORKDIR oracle-suite
RUN git checkout 9f313759357d51b831713dd4802d328741306c93
RUN go build -o /bin/gofer ./cmd/gofer

FROM rust:1.69-alpine
WORKDIR /usr/src/oracle-client
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./config.hcl ./config.hcl

RUN apk add musl-dev
RUN apk add openssl-dev
RUN apk add pkgconfig openssl


# RUN cargo build --target=x86_64-unknown-linux-musl --release

ENV RUST_LOG=debug
ENV GOFER_CMD=/bin/gofer
ENTRYPOINT ["cargo", "run"]


