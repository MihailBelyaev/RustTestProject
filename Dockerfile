FROM rust:latest as build
ENV TARGET=x86_64-unknown-linux-musl

#RUN rustup target add ${TARGET}

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target-dir /usr/local/cargo/bin/RustTestProject/

FROM debian:buster-slim as runtime
RUN apt-get update &&\
    apt-get install -y&&\
    rm -rf /var/lib/apt/lists/*
    

COPY --from=build /usr/local/cargo/bin/RustTestProject/release/RustTestProject /usr/local/bin/RustTestProject
ENTRYPOINT ["RustTestProject"]