SHELL := /bin/sh

ROOT_DIR:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

debug-%: ; @echo $*=$($*)

.PHONY: all debug local release mock-node fmt clean

.DEFAULT: all

all: debug mock-node

local: debug-local mock-node-local
	
debug:
	cargo build

debug-local:
	cargo build --features "local" --no-default-features 

release:
	cargo build --release

mock-node:
	cd sim/mock-node && cargo build --target-dir=$(ROOT_DIR)/target

mock-node-local:
	cd sim/mock-node && cargo build --features "local" --no-default-features --target-dir=$(ROOT_DIR)/target

touch: 
	cargo update -p https://github.com/prrkl/mothra#0.1.0
	cd sim/mock-node && cargo update -p https://github.com/prrkl/mothra#0.1.0

fmt:
	cargo fmt
	cd agent && cargo fmt
	cd eth2 && cargo fmt
	cd network && cargo fmt
	cd network/p2p && cargo fmt
	cd sim/mock-node && cargo fmt
	cd types && cargo fmt
	cd utils && cargo fmt

clean:
	cargo clean
	cd sim/mock-node && cargo clean --target-dir=$(ROOT_DIR)/target