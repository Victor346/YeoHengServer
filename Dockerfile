FROM rust:latest as builder
COPY . .
RUN cargo build --release
WORKDIR /target/release
FROM debian:stable
RUN apt-get update
COPY --from=builder /target/release .
CMD ["./yeoheng-server"]