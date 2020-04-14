SHELL := /bin/sh

.PHONY: all debug local release mock-node fmt clean

.DEFAULT: all

all: debug mock-node

debug:
	cargo build

local:
	cargo build --features "local" --no-default-features

release:
	cargo build --release

mock-node:
	cd sim/mock-node && cargo build

fmt:
	cargo fmt
	cd agent && cargo fmt
	cd datatypes && cargo fmt
	cd network && cargo fmt
	cd network/p2p && cargo fmt
	cd sim/mock-node && cargo fmt

clean:
	cargo clean
	cd sim-mock-node && cargo clean