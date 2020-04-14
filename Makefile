SHELL := /bin/sh

.PHONY: debug local release clean

.DEFAULT: debug

debug:
	cargo build

local:
	cargo build --features "local" --no-default-features

release:
	cargo build --release

fmt:
	cargo fmt
	cd agent && cargo fmt
	cd datatypes && cargo fmt
	cd network && cargo fmt
	cd network/p2p && cargo fmt

clean:
	cargo clean