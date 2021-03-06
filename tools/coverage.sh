#!/usr/bin/env bash

set -euo pipefail


export RUSTFLAGS="-Zinstrument-coverage"
export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"
rustup default nightly-2021-10-21
rustup toolchain list
cargo +nightly-2021-10-21 install grcov
rustup component add llvm-tools-preview
brew install lcov

cargo test --all --tests --no-fail-fast --all-features -- -Z unstable-options
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing  --ignore "*debug*"  --ignore "*cargo*" -o ./coverage/
grcov . --binary-path ./target/debug/ -s . -t lcov --llvm --branch --ignore-not-existing --ignore "*debug*" --ignore "*cargo*" -o coverage.lcov
lcov --summary coverage.lcov