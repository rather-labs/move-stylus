disassemble:
	cargo run -p move-cli -- disassemble --name hello_world -p ./example --Xdebug

check:
	cargo stylus check --wasm-file=./example/out.wasm --endpoint https://arb1.arbitrum.io/rpc

build:
	cargo run -p move-cli -- build -p ./example

setup:
	RUSTFLAGS="-C link-args=-rdynamic" cargo install --force cargo-stylus
