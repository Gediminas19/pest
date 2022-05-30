FROM fuzzers/cargo-fuzz:0.10.0 as builder

ADD . /pest
WORKDIR /pest
RUN rustup toolchain install nightly
WORKDIR /pest/meta
RUN cargo bootstrap && RUSTFLAGS="-Znew-llvm-pass-manager=no" cargo +nightly fuzz build
WORKDIR /pest/grammars
RUN cargo bootstrap && RUSTFLAGS="-Znew-llvm-pass-manager=no" cargo +nightly fuzz build

FROM ubuntu:20.04
COPY --from=builder /pest/meta/fuzz/target/x86_64-unknown-linux-gnu/release /
