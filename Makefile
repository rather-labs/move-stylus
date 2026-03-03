PRIVATE_KEY ?= 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
CONTRACT_NAME ?= hello_world
CONTRACT_ENV_VAR ?= CONTRACT_ADDRESS
PACKAGE_VERSION=$(shell cat ./crates/move-cli/Cargo.toml | grep version | head -n 1 | awk '{print $$3}' | sed -e 's/"//g')

test-move-bytecode-to-wasm:
	cargo test -p move-bytecode-to-wasm

test:
	cargo test

disassemble:
	cargo run -p move-stylus -- disassemble --name hello_world -p ./example --Xdebug

unit-test:
	cargo run -p move-stylus -- test -p ./example

disassemble-module:
	cargo run -p move-stylus -- disassemble --name $(filter-out $@,$(MAKECMDGOALS)) -p ./example --Xdebug
%:
	@:

build-example:
	cargo run -p move-stylus -- build -p ./example

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

example-all:
	$(MAKE) example-interaction
	$(MAKE) example-interaction-2
	$(MAKE) example-interaction-primitives
	$(MAKE) example-counter
	$(MAKE) example-counter-named-id
	$(MAKE) example-counter-with-init
	$(MAKE) example-dog-walker
	$(MAKE) example-erc20
	$(MAKE) example-erc721
	$(MAKE) example-cross-contract-call
	$(MAKE) example-delegated-counter
	$(MAKE) example-delegated-counter-named-id
	$(MAKE) example-revert-errors

deploy:
	cargo run -p move-stylus -- deploy -p ./example \
		--endpoint 'http://localhost:8547' \
		--private-key "$(PRIVATE_KEY)" \
		--contract-name "$(CONTRACT_NAME)" \
		| tee /dev/tty | ./update_contract_env.sh $(CONTRACT_ENV_VAR)

deploy-example:
	$(MAKE) deploy CONTRACT_NAME=hello_world CONTRACT_ENV_VAR=CONTRACT_ADDRESS

deploy-example-2:
	$(MAKE) deploy CONTRACT_NAME=hello_world_2 CONTRACT_ENV_VAR=CONTRACT_ADDRESS_2

deploy-example-primitives:
	$(MAKE) deploy CONTRACT_NAME=primitives_and_operations CONTRACT_ENV_VAR=CONTRACT_ADDRESS_PRIMITIVES

deploy-erc20:
	$(MAKE) deploy CONTRACT_NAME=erc20 CONTRACT_ENV_VAR=CONTRACT_ADDRESS_ERC20

deploy-erc721:
	$(MAKE) deploy CONTRACT_NAME=erc721 CONTRACT_ENV_VAR=CONTRACT_ADDRESS_ERC721
	$(MAKE) deploy CONTRACT_NAME=erc721_receiver CONTRACT_ENV_VAR=CONTRACT_ADDRESS_ERC721_RECEIVER

deploy-counter:
	$(MAKE) deploy CONTRACT_NAME=counter CONTRACT_ENV_VAR=CONTRACT_ADDRESS_COUNTER

deploy-counter-named-id:
	$(MAKE) deploy CONTRACT_NAME=counter_named_id CONTRACT_ENV_VAR=CONTRACT_ADDRESS_COUNTER_NAMED_ID

deploy-counter-with-init:
	$(MAKE) deploy CONTRACT_NAME=counter_with_init CONTRACT_ENV_VAR=CONTRACT_ADDRESS_COUNTER_WITH_INIT

deploy-dog-walker:
	$(MAKE) deploy CONTRACT_NAME=dog_walker CONTRACT_ENV_VAR=CONTRACT_ADDRESS_DOG_WALKER

deploy-cross-contract-call:
	$(MAKE) deploy CONTRACT_NAME=cross_contract_call CONTRACT_ENV_VAR=CONTRACT_ADDRESS_CROSS_CALL

deploy-delegated-counter:
	$(MAKE) deploy CONTRACT_NAME=delegated_counter_logic_1 CONTRACT_ENV_VAR=CONTRACT_ADDRESS_DELEGATED_COUNTER_LOGIC_1
	$(MAKE) deploy CONTRACT_NAME=delegated_counter_logic_2 CONTRACT_ENV_VAR=CONTRACT_ADDRESS_DELEGATED_COUNTER_LOGIC_2
	$(MAKE) deploy CONTRACT_NAME=delegated_counter CONTRACT_ENV_VAR=CONTRACT_ADDRESS_DELEGATED_COUNTER

deploy-delegated-counter-named-id:
	$(MAKE) deploy CONTRACT_NAME=delegated_counter_named_id_logic_1 CONTRACT_ENV_VAR=CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID_LOGIC_1
	$(MAKE) deploy CONTRACT_NAME=delegated_counter_named_id_logic_2 CONTRACT_ENV_VAR=CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID_LOGIC_2
	$(MAKE) deploy CONTRACT_NAME=delegated_counter_named_id CONTRACT_ENV_VAR=CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID

deploy-revert-errors:
	$(MAKE) deploy CONTRACT_NAME=revert_errors CONTRACT_ENV_VAR=CONTRACT_ADDRESS_REVERT_ERRORS

deploy-all:
	$(MAKE) deploy-example
	$(MAKE) deploy-example-2
	$(MAKE) deploy-example-primitives
	$(MAKE) deploy-erc20
	$(MAKE) deploy-erc721
	$(MAKE) deploy-counter
	$(MAKE) deploy-counter-named-id
	$(MAKE) deploy-counter-with-init
	$(MAKE) deploy-dog-walker
	$(MAKE) deploy-cross-contract-call
	$(MAKE) deploy-delegated-counter
	$(MAKE) deploy-delegated-counter-named-id
	$(MAKE) deploy-revert-errors

install-wasm-tools:
	cargo install --locked wasm-tools

install:
	cargo install --locked --path crates/move-cli --force

release-x86_64:
	rustup target add x86_64-unknown-linux-gnu
	CC=x86_64-linux-gnu-gcc \
		CXX=x86_64-linux-gnu-g++ \
		RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc' \
		cargo build --release -p move-stylus --target x86_64-unknown-linux-gnu
	mkdir -p dist
	tar -czvf ./dist/move-stylus-x86_64-linux-$(PACKAGE_VERSION).tar.gz -C target/x86_64-unknown-linux-gnu/release move-stylus
	sha256sum ./dist/move-stylus-x86_64-linux-$(PACKAGE_VERSION).tar.gz > ./dist/move-stylus-x86_64-linux-$(PACKAGE_VERSION).tar.gz.sha256

release-aarch64_linux:
	rustup target add aarch64-unknown-linux-gnu
	CC=aarch64-linux-gnu-gcc \
		CXX=aarch64-linux-gnu-g++ \
		RUSTFLAGS='-C linker=aarch64-linux-gnu-gcc' \
		cargo build --release -p move-stylus --target aarch64-unknown-linux-gnu
	mkdir -p dist
	tar -czvf ./dist/move-stylus-aarch64-linux-$(PACKAGE_VERSION).tar.gz -C target/aarch64-unknown-linux-gnu/release move-stylus
	sha256sum ./dist/move-stylus-aarch64-linux-$(PACKAGE_VERSION).tar.gz > ./dist/move-stylus-aarch64-linux-$(PACKAGE_VERSION).tar.gz.sha256

release-macos:
	cargo build --release -p move-stylus --target aarch64-apple-darwin
	mkdir -p dist
	tar -czvf ./dist/move-stylus-aarch64-macos-$(PACKAGE_VERSION).tar.gz -C target/aarch64-apple-darwin/release move-stylus
	sha256sum ./dist/move-stylus-aarch64-macos-$(PACKAGE_VERSION).tar.gz > ./dist/move-stylus-aarch64-macos-$(PACKAGE_VERSION).tar.gz.sha256

release-linux:
	$(MAKE) release-x86_64
	$(MAKE) release-aarch64_linux

.PHONY: test setup-stylus install deploy-* example-* disassemble* unit-test check-example build-example release-aarch64_macos release-aarch64_linux release-x86_64 release-all
