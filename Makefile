.PHONY: all build run

.default: build

build:
	cargo build

all: build
