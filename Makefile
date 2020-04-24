SHELL := /bin/sh

ROOT_DIR:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

debug-%: ; @echo $*=$($*)

.PHONY: all debug local release fmt clean

.DEFAULT: all

all: debug 

local: debug-local 
	
debug:
	cargo build

debug-local:
	cargo build --features "local" --no-default-features 

release:
	cargo build --release


touch: 
	cargo update -p https://github.com/prrkl/mothra#0.1.0
	cargo update -p tree_hash
	cargo update -p tree_hash_derive
	cargo update -p eth2_ssz
	cargo update -p eth2_ssz_derive
	cargo update -p eth2_ssz_types
	cargo update -p types
	cargo update -p eth2_hashing
	cargo update -p eth2-libp2p
	cargo update -p eth2_config
	cargo update -p eth2_testnet_config

fmt:
	cargo fmt
	cd agent && cargo fmt
	cd eth2 && cargo fmt
	cd network && cargo fmt
	cd network/p2p && cargo fmt
	cd types && cargo fmt
	cd utils && cargo fmt

clean:
	cargo clean