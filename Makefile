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