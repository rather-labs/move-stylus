disassemble:
	cargo run -p move-cli -- disassemble --name hello_world -p ./example --Xdebug

check:
	cargo stylus check --wasm-file=./example/build/wasm/output.wasm --endpoint https://arb1.arbitrum.io/rpc

build:
	cargo run -p move-cli -- build -p ./example

setup-stylus:
	RUSTFLAGS="-C link-args=-rdynamic" cargo install --force cargo-stylus

install-wasm-tools:
	cargo install --locked wasm-tools

parse-rust-example:
	wasm-tools print ./example-rust/target/wasm32-unknown-unknown/release/stylus_hello_world.wasm -o ./example-rust/target/wasm32-unknown-unknown/release/stylus_hello_world.wat
