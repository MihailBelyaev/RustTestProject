FROM rust:latest as build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target-dir /usr/local/cargo/bin/RustTestProject/

FROM debian:buster-slim as runtime    
ENV RUST_LOG=debug
COPY --from=build /usr/local/cargo/bin/RustTestProject/release/RustTestProject /usr/local/bin/RustTestProject
EXPOSE 3030
ENTRYPOINT ["RustTestProject"]