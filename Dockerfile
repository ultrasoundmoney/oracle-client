# Install gofer
FROM golang:1.20-bullseye as go
RUN apt install git
RUN git clone https://github.com/chronicleprotocol/oracle-suite.git
WORKDIR oracle-suite
RUN git checkout 9f313759357d51b831713dd4802d328741306c93
RUN go build -o /app/gofer ./cmd/gofer


FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
# Figure out if dependencies have changed.
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this layer is cached for massive speed up.
RUN cargo chef cook --release --recipe-path recipe.json
# Build application - this should be re-done every time we update our src.
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app
# sqlx depends on native TLS, which is missing in buster-slim.
RUN apt update && apt install -y libssl1.1 ca-certificates
COPY --from=builder /app/target/release/oracle-client /usr/local/bin
COPY --from=go /app/gofer /usr/local/bin
COPY ./config.hcl ./config.hcl
ENV RUST_LOG=info
ENV GOFER_CMD=/usr/local/bin/gofer
ENV SERVER_URL=http://host.docker.internal:3000/post_oracle_message

ENTRYPOINT ["/usr/local/bin/oracle-client"]


