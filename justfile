run:
    RUST_LOG='debug' RUST_BACKTRACE=1 cargo run
run-trace:
    RUST_LOG='debug,node=trace,gui=trace' RUST_BACKTRACE=1 cargo run

check:
    cargo clippy
test:
    cargo test
build:
    cargo build
build-release:
    cargo build --release
