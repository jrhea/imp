SHELL := /bin/sh

ROOT_DIR:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

debug-%: ; @echo $*=$($*)

.PHONY: all debug local release docker touch fmt clean

.DEFAULT: all

all: debug 

local: debug-local 
	
debug:
	cargo build

debug-local:
	cargo build --features "local" --no-default-features 

debug-docker: 
	docker run --rm --user "$$(id -u)":"$$(id -g)" -v "$$PWD":/usr/src/imp -w /usr/src/imp -it rust:latest bash -c "cargo build"

release:
	cargo build --release

release-docker: 
	docker run --rm --user "$$(id -u)":"$$(id -g)" -v "$$PWD":/usr/src/imp -w /usr/src/imp -it rust:latest bash -c "cargo build --release"

crawl-docker:
	docker run --rm --user "$$(id -u)":"$$(id -g)" -v "$$PWD":/usr/src/imp -w /usr/src/imp -it rust:latest bash -c "cd scripts && bash crawl-network.sh witti 1 snapshot"

enr-count-docker:
	docker run --rm --user "$$(id -u)":"$$(id -g)" -v "$$PWD":/usr/src/imp -w /usr/src/imp -it rust:latest bash -c "tail -n+2 .witti/crawler* | grep f6775d07 | sed 's/\".*\"//g' |  cut -d',' -f3,12,14 | sort -t ',' -k1,1 -k2,2nr -s -u | sort -t ',' -u -k1,1 | cut -d',' -f3 |sed -e "s/^enr://" | wc -l"

touch: 
	cargo update -p https://github.com/prrkl/mothra#0.1.0
	cargo update -p discv5
	cargo update -p tree_hash
	cargo update -p tree_hash_derive
	cargo update -p eth2_ssz
	cargo update -p eth2_ssz_derive
	cargo update -p eth2_ssz_types
	cargo update -p types
	cargo update -p eth2_hashing
	cargo update -p eth2_libp2p
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