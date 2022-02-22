# Memo

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen .\target\wasm32-unknown-unknown\release\bevy_ggrs_demo.wasm --target web --out-dir docs --no-typescript
