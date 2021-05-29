.PHONY: all build run test

TARGETS:="armv7-unknown-linux-gnueabihf"
BIN="rpw"


.default: build

build:
	cargo build

release:
	cargo build --release
	@for t in ${TARGETS}; 				\
	do 						\
		echo "Building $$t"			\
		cargo b --release --target=$$t; 	\
	done
	@for b in `find target -name "rpw"`; \
	do						\
		echo "Releaseing $$b --> ?";		\
	done



install: release
	ln -svf `realpath ./target/release/rpw` `realpath ~/bin`

test:
	cargo test

all: build
