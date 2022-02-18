FROM rust:latest as build
ENV TARGET=x86_64-unknown-linux-musl

RUN rustup target add ${TARGET}

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target=${TARGET}

FROM scratch
COPY --from=build /usr/local/cargo/bin/RustTestProject .
USER 1000
CMD ["./RustTestProject"]