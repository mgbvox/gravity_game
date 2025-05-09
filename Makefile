HERE:=$(shell pwd)
WASM_DIR:=$(HERE)/target/wasm32-unknown-unknown
WASM_RELEASE_DIR:=$(WASM_DIR)/release/
#
#wasm:
#	cargo build --release --target wasm32-unknown-unknown
#	wasm-opt -O4 -o $(WASM_RELEASE_FILE).o4 $(WASM_RELEASE_FILE)

wasm:
	cargo build --release --target wasm32-unknown-unknown
	wasm-opt -O -o $(WASM_RELEASE_DIR)/gravity_game_o1.wasm $(WASM_RELEASE_DIR)/gravity_game.wasm