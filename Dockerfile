FROM rust:1.40.0 as builder
RUN apt-get update && apt-get install -y musl-dev
RUN rustup target add x86_64-unknown-linux-musl
ENV CARGO_INSTALL_ROOT /opt/rust-tools
ADD . /cpubars
RUN cargo install --target x86_64-unknown-linux-musl --path /cpubars

FROM alpine:3.11
COPY --from=builder /opt/rust-tools/bin/* /opt/rust-tools/bin/
