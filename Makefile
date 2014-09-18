all: build

build:
	mkdir -p build
	rustc --out-dir build --test src/lib.rs
	build/kademlia

.PHONY: all build
