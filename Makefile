test-move-bytecode-to-wasm:
	cargo test -p move-bytecode-to-wasm

disassemble:
	cargo run -p move-cli -- disassemble --name hello_world -p ./example --Xdebug

check-example:
	cargo stylus check --wasm-file=./example/build/wasm/hello_world.wasm --endpoint http://127.0.0.1:8547

build-example:
	cargo run -p move-cli -- build -p ./example

example-interaction:
	cargo run -p move-hello-world-example --bin interaction

deploy-example:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/wasm/hello_world.wasm \
		--no-verify

setup-stylus:
	RUSTFLAGS="-C link-args=-rdynamic" cargo install --force cargo-stylus

install-wasm-tools:
	cargo install --locked wasm-tools

parse-rust-example:
	wasm-tools print ./example-rust/target/wasm32-unknown-unknown/release/stylus_hello_world.wasm -o ./example-rust/target/wasm32-unknown-unknown/release/stylus_hello_world.wat

# This is used to compile solidity contracts and obtain a compiled abi.
# It is useful for contracts that have complex datatypes such as enums or structs, because abigen!
# can't parse them.
# Usage = make compile-sol-abi SOL_SRC=./crates/move-bytecode-to-wasm/tests/structs/unpacking_struct.sol
SOL_SRC_ABS := $(realpath $(SOL_SRC))
SOL_SRC_DIR := $(dir $(SOL_SRC_ABS))
SOL_SRC_FILE := $(notdir $(SOL_SRC_ABS))
compile-sol-abi:
	docker run --rm \
		--volume "$(SOL_SRC_DIR):/sources" \
		ethereum/solc:stable \
		/sources/$(SOL_SRC_FILE) \
		--abi \
		--overwrite \
		--output-dir /sources
