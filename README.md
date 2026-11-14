## Minimal Wasm setup

`web` builds a `cdylib`. In `web/src/lib.rs`,
`#[wasm_bindgen]` on `Viewer` exports a JavaScript class; the same attribute on
`impl Viewer` exports its public methods. `#[wasm_bindgen(constructor)]` enables
`new Viewer()`. Private fields and helper methods stay internal.

Run these commands in PowerShell from the repository root. One-time setup:

```powershell
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo +nightly install wasm-bindgen-cli --version 0.2.130 --locked
```

The `node` crate uses nightly Rust features. The CLI version must match the
resolved `wasm-bindgen` crate version (currently `0.2.130`).

Build and serve:

```powershell
just build
just host
```

`just build` compiles Rust to Wasm and generates JavaScript bindings, the Wasm
binary, and TypeScript declarations in `web/static/gen`. `just host` serves
`web/static`, explicitly mapping `.js` to `text/javascript` and `.wasm` to
`application/wasm` so system MIME mappings do not prevent module loading.
Open <http://127.0.0.1:8000> to see the canvas example. Stop the server with
Ctrl+C. After Rust changes, rerun `just build` and refresh the page.

`web/static/index.js` shows the minimal usage: import `init` and `Viewer`,
`await init()`, then call `new Viewer()` and its methods.
`set_step` takes a JavaScript `bigint` (`0n`); canvas size, center, and period
take JavaScript numbers. `set_period(undefined)` or `set_period(null)` disables
the period; `set_period(4)` sets it, and `0` throws. Methods returning `Result`
throw on `Err`. `get_rgba()` returns a
`Uint8Array`. Call `viewer.free()` when finished with the instance.

## GUI help
- can move display by dragging with right mouse button
- can zoom in with `I` and out with `O` when focused on board
  - center of board will stay in the same location
- can step once with `Space`
  - has a base step size that currently cannot be configured
- can play/pause with `Enter`
  - can increase speed with `+` and decrease with `-`
  - can be less than one step per frame, in which case it delays based on real time
- cursor modes: (only when zoomed in to see individual cells)
  - `Escape` - select area with left mouse button
    - `C` - copy selected area into clipboard
    - `D` - clear selected area
    - `X` - copy and delete selected area
  - `E` - toggle cells with left mouse click
  - `V` - paste clipboard with left mouse click
    - `R` - rotate CW, `Shift+R` to rotate CCW
    - `F` - flip horizontally, `Shift+F` to flip vertically
