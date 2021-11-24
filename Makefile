APP = sketch_assist

run:
	cargo run --release --features native

web_build:
	cargo build --release --target wasm32-unknown-unknown --features web
	wasm-bindgen --out-dir target --out-name wasm --target web --no-typescript target/wasm32-unknown-unknown/release/$(APP).wasm

serve: web_build
	basic-http-server -x
