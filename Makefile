.DEFAULT_GOAL := all

OUT_H = ./x/async_rt.h

.PHONY: all

all:
	cargo fmt && cargo clippy -- -D warnings && cargo build --release
	cbindgen --lang c -o ${OUT_H}
	sed -i 's/typedef struct Awaitable Awaitable;/typedef struct Awaitable {} Awaitable;/g' ${OUT_H}
	clang-format -i ${OUT_H}
