FROM fuzzers/cargo-fuzz:0.10.0

ADD . /pest
WORKDIR /pest
RUN rustup toolchain install nightly
WORKDIR /pest/meta
RUN cargo bootstrap && RUSTFLAGS="-Znew-llvm-pass-manager=no" cargo +nightly fuzz build
WORKDIR /pest/grammars
RUN cargo bootstrap && RUSTFLAGS="-Znew-llvm-pass-manager=no" cargo +nightly fuzz build

# Set to fuzz!
ENTRYPOINT []
