FROM rust:latest as builder
COPY . .
RUN cargo build --release
WORKDIR /target/release
FROM debian:stable
RUN apt-get update
RUN apt-get -y install libssl-dev
COPY --from=builder /target/release/yeoheng-server .
CMD ["./yeoheng-server"]