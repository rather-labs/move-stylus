test-move-bytecode-to-wasm:
	cargo test -p move-bytecode-to-wasm

test:
	cargo test

disassemble:
	cargo run -p move-cli -- disassemble --name hello_world -p ./example --Xdebug

unit-test:
	cargo run -p move-cli -- test -p ./example

disassemble-module:
	cargo run -p move-cli -- disassemble --name $(filter-out $@,$(MAKECMDGOALS)) -p ./example --Xdebug
%:
	@:

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

example-counter-named-id:
	cargo run -p move-hello-world-example --bin counter_named_id

example-counter-with-init:
	cargo run -p move-hello-world-example --bin counter_with_init

example-dog-walker:
	cargo run -p move-hello-world-example --bin dog_walker

example-erc20:
	cargo run -p move-hello-world-example --bin erc20

example-erc721:
	cargo run -p move-hello-world-example --bin erc721

example-cross-contract-call:
	cargo run -p move-hello-world-example --bin cross_contract_call

example-delegated-counter:
	cargo run -p move-hello-world-example --bin delegated_counter

example-delegated-counter-named-id:
	cargo run -p move-hello-world-example --bin delegated_counter_named_id

example-revert-errors:
	cargo run -p move-hello-world-example --bin revert_errors

deploy-example:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/hello_world.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS

deploy-example-2:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/hello_world_2.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_2

deploy-example-primitives:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/primitives_and_operations.wasm \
		--no-verify \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_PRIMITIVES

deploy-erc20:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/erc20.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_ERC20

deploy-erc721:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/erc721.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_ERC721
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/erc721_receiver.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_ERC721_RECEIVER

deploy-counter:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/counter.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_COUNTER

deploy-counter-named-id:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/counter_named_id.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_COUNTER_NAMED_ID

deploy-counter-with-init:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/counter_with_init.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_COUNTER_WITH_INIT

deploy-dog-walker:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/dog_walker.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_DOG_WALKER

deploy-cross-contract-call:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/cross_contract_call.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_CROSS_CALL

deploy-delegated-counter:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/delegated_counter_logic_1.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_DELEGATED_COUNTER_LOGIC_1
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/delegated_counter_logic_2.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_DELEGATED_COUNTER_LOGIC_2
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/delegated_counter.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_DELEGATED_COUNTER

deploy-delegated-counter-named-id:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/delegated_counter_named_id_logic_1.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID_LOGIC_1
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/delegated_counter_named_id_logic_2.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID_LOGIC_2
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/delegated_counter_named_id.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID

deploy-revert-errors:
	cargo stylus deploy \
		--endpoint='http://localhost:8547' \
		--private-key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" \
		--wasm-file=./example/build/example/wasm/revert_errors.wasm \
		| tee /dev/tty | ./update_contract_env.sh CONTRACT_ADDRESS_REVERT_ERRORS

setup-stylus:
	RUSTFLAGS="-C link-args=-rdynamic" cargo install --force --version 0.6.3 cargo-stylus

install-wasm-tools:
	cargo install --locked wasm-tools

parse-rust-example:
	wasm-tools print ./example-rust/target/wasm32-unknown-unknown/release/stylus_hello_world.wasm -o ./example-rust/target/wasm32-unknown-unknown/release/stylus_hello_world.wat

install:
	@which cargo-stylus > /dev/null || (echo "Installing mdbook..." && $(MAKE) setup-stylus)
	cargo install --locked --path crates/move-cli

.PHONY: test setup-stylus install deploy-*
