.DEFAULT_GOAL := all

OUT_H       = ./x/async_rt.h
GEN_PATH    = ./x/async/src/Async/Generated.idr
LIB_PREFIX ?= ${HOME}/.idris2/lib

.PHONY: all rlib ilib

all: rlib ilib

rlib:
	(cargo fmt && cargo clippy -- -D warnings && cargo build --release)
	(cbindgen --lang c -o ${OUT_H})
	(sed -i 's/typedef struct Awaitable Awaitable;/typedef struct Awaitable {} Awaitable;/g' ${OUT_H})
	(clang-format -i ${OUT_H})

ilib:
	(echo '-- @generated' > ${GEN_PATH} && \
		echo 'module Async.Generated'                                          >> ${GEN_PATH} && \
		echo '%default total'                                                  >> ${GEN_PATH} && \
		echo ''                                                                >> ${GEN_PATH} && \
		echo 'public export'                                                   >> ${GEN_PATH} && \
		echo 'rtLib : String -> String'                                        >> ${GEN_PATH} && \
		echo "rtLib f = \"C:\" ++ f ++ \", ${LIB_PREFIX}/libasyncrt.so\"" >> ${GEN_PATH})
	(cp target/release/libasyncrt.so ${LIB_PREFIX})
	(cd x/async && idris2 --build async.ipkg)

run:
	(./x/async/build/exec/main)
