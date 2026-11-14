run: build host

check:
    cargo clippy

test:
    cargo test

build:
    cargo build --release --target wasm32-unknown-unknown -p web
    wasm-bindgen --target web --out-dir web/static/gen --out-name lib target/wasm32-unknown-unknown/release/web.wasm

build-dev:
    cargo build --target wasm32-unknown-unknown -p web
    wasm-bindgen --target web --keep-debug --out-dir web/static/gen --out-name lib target/wasm32-unknown-unknown/debug/web.wasm

host:
    miniserve web/static --index index.html -i 127.0.0.1 -p 8000

install-dev:
    cargo install miniserve
    cargo install wasm-bindgen-cli@0.2.130
