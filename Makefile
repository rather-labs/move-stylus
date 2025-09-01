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

example-interaction-2:
	cargo run -p move-hello-world-example --bin interaction_2

example-interaction-primitives:
	cargo run -p move-hello-world-example --bin primitives_and_operations

example-counter:
	cargo run -p move-hello-world-example --bin counter

example-counter-with-init:
	cargo run -p move-hello-world-example --bin counter_with_init

example-dog-walker:
	cargo run -p move-hello-world-example --bin dog_walker

deploy-example:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/wasm/hello_world.wasm

deploy-example-2:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/wasm/hello_world_2.wasm

deploy-example-primitives:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/wasm/primitives_and_operations.wasm \
		--no-verify

deploy-counter:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/wasm/counter.wasm

deploy-counter-with-init:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/wasm/counter_with_init.wasm

deploy-dog-walker:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/wasm/dog_walker.wasm

setup-stylus:
	RUSTFLAGS="-C link-args=-rdynamic" cargo install --force cargo-stylus

install-wasm-tools:
	cargo install --locked wasm-tools

parse-rust-example:
	wasm-tools print ./example-rust/target/wasm32-unknown-unknown/release/stylus_hello_world.wasm -o ./example-rust/target/wasm32-unknown-unknown/release/stylus_hello_world.wat
