run: build host

check:
    cargo clippy

test:
    cargo test

build:
    cargo build --release --target wasm32-unknown-unknown -p web
    wasm-bindgen --target web --out-dir web/gen --out-name lib target/wasm32-unknown-unknown/release/web.wasm
    npm ci
    npx tsc -p web/tsconfig.json
    npx esbuild web/src/index.ts --bundle --minify --format=esm --outfile=web/static/bundle.js
    # esbuild doesn't pick up dynamic import of wasm files
    cp web/gen/lib_bg.wasm web/static

build-dev:
    cargo build --target wasm32-unknown-unknown -p web
    wasm-bindgen --target web --keep-debug --out-dir web/gen --out-name lib target/wasm32-unknown-unknown/debug/web.wasm
    npm ci
    npx tsc -p web/tsconfig.json
    npx esbuild web/src/index.ts --bundle --sourcemap=inline --format=esm --outfile=web/static/bundle.js
    # esbuild doesn't pick up dynamic import of wasm files
    cp web/gen/lib_bg.wasm web/static

host:
    miniserve web/static --index index.html -i 127.0.0.1 -p 8000

install-tooling:
    cargo install miniserve
    cargo install wasm-bindgen-cli@0.2.130
