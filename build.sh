cargo build --target wasm32-unknown-unknown --release
~/.cargo/bin/wasm-bindgen --out-dir ./out --target web ./target/wasm32-unknown-unknown/release/game_off_2025.wasm
