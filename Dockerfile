FROM rust:1.68.1-alpine AS builder

RUN mkdir -p /usr/src/deviceservice
WORKDIR /usr/src/deviceservice

COPY src src
COPY main.rs main.rs
COPY Cargo.toml .
COPY LICENSE .

RUN apk update
RUN apk add pkgconfig openssl openssl-dev musl-dev
RUN rustup toolchain install beta
RUN rustup default beta

RUN cargo build --target x86_64-unknown-linux-musl --release

EXPOSE 8080

CMD ["target/release/deviceservice"]


FROM scratch

COPY --from=builder /usr/src/deviceservice/target/x86_64-unknown-linux-musl/release/deviceservice /main

EXPOSE 8080

CMD ["/main"]
