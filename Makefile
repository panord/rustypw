.PHONY: all build run test

TARGETS:="armv7-unknown-linux-gnueabihf"
BIN="rpw"

.default: build

build:
	cargo build

release:
	@echo "Building";				\
	echo -e "$$(uname -m)";				\
	cargo build --release;				\
	for t in ${TARGETS}; 				\
	do 						\
		echo -e "$$t";				\
		cargo b --release --target=$$t; 	\
	done

upload: release
	@mkdir ".release" 2>/dev/null;					\
	for b in `find target -name "rpw"`;				\
	do								\
		source="$$b";						\
		arch="$$(basename $$(dirname $$(dirname $$b)))";	\
		dir=".release";						\
		if [[ $$arch != "target" ]]; then 			\
			target="$${dir}/$$(basename $$b)-$${arch}";	\
		else 							\
			target="$${dir}/$$(basename $$b)-$$(uname -m)";	\
		fi;							\
		cp $${source} $${target};				\
	done
	@if [[ "${SCPDIR}" != "" ]]; then			\
		echo "Uploading release(s) to ${SCPDIR}";	\
		for f in $$(find ".release" -type f);		\
		do						\
			echo -e "\treleasing $$f";		\
			scp $$f ${SCPDIR};			\
		done						\
	fi;							\

install: release
	ln -svf `realpath ./target/release/rpw` `realpath ~/bin`

test:
	cargo test

clean:
	cargo clean

all: build
