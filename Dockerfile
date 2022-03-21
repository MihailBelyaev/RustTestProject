FROM rust:1.58 as lint
COPY Cargo.toml Cargo.lock ./app/
COPY src ./app/src
WORKDIR /app
RUN rustup component add rustfmt clippy
RUN cargo fmt --all
RUN cargo clippy --all --tests --examples

FROM lint as build
RUN cargo build --release --target-dir /usr/local/cargo/bin/RustTestProject/

# todo move testcontainers to features for run only unit tests
FROM build as test
RUN cargo test --all --tests --no-fail-fast -- -Z unstable-options
ENTRYPOINT ["cargo", "test", "--all", "--tests", "--no-fail-fast", "--", "-Z", "unstable-options"]

FROM build as coverage
ENV RUSTFLAGS="-Zinstrument-coverage"
ENV LLVM_PROFILE_FILE="coverage-%p-%m.profraw"
ENV CARGO_INCREMENTAL=0
RUN apt update && apt install -y lcov
RUN rustup default nightly-2021-10-21
RUN cargo +nightly-2021-10-21 install grcov
RUN rustup +nightly-2021-10-21 component add llvm-tools-preview
WORKDIR /app
RUN cargo test +nightly-2021-10-21 --all --tests --examples -- -Z unstable-options
RUN grcov . --binary-path ./target/debug/ -s . \
        -t lcov --llvm --branch --ignore-not-existing \
        --ignore "*cargo*" --ignore "*example*" --ignore "*debug*" --ignore "*main*" \
        -o target/coverage.lcov
RUN lcov --summary target/coverage.lcov
ENTRYPOINT ["lcov", "--summary target/coverage.lcov"]

FROM ubuntu:22.04 as runtime
ENV RUST_LOG=debug
COPY --from=build /usr/local/cargo/bin/RustTestProject/release/rust_test_project /usr/local/bin/app
EXPOSE 3030

RUN apt update && apt install sqlite3 -y
RUN apt-get install libc6-dev -y && \
    apt-get clean autoclean && \
    apt-get autoremove --yes  && \
    rm -rf /var/lib/{apt,dpkg,cache,log}/
ENTRYPOINT ["app"]

