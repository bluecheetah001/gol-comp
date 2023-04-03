run:
    RUST_LOG='trace,eframe=debug' cargo run

check:
    cargo clippy
test:
    cargo test
build:
    cargo build
build-release:
    cargo build --release
