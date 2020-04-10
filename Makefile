SHELL := /bin/sh

.PHONY: debug local release clean

.DEFAULT: debug

debug:
	cargo build

local:
	cargo build --features "local" --no-default-features

release:
	cargo build --release

clean:
	cargo clean