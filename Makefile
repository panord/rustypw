.PHONY: all build run

.default: build

build:
	cargo build

release:
	cargo build --release

install: release
	ln -svf `realpath ./target/release/rpw` `realpath ~/bin`/rpw

all: build
