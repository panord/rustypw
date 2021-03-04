.PHONY: all build run test

.default: build

build:
	cargo build

release:
	cargo build --release

install: release
	ln -svf `realpath ./target/release/rpw` `realpath ~/bin`

test:
	cargo test

all: build
