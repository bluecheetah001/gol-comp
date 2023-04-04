run:
    RUST_LOG='debug' cargo run
run-trace:
    RUST_LOG='debug,node=trace,gui=trace' cargo run

check:
    cargo clippy
test:
    cargo test
build:
    cargo build
build-release:
    cargo build --release
