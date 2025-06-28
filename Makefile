BIN=webrs
RUST_TARGET=x86_64-unknown-linux-musl

all: build

build:
	RUSTFLAGS="-C target-cpu=znver2" cargo build --target $(RUST_TARGET) --release

release_run: build
	./target/$(RUST_TARGET)/release/$(BIN)

clean:
	cargo clean
	rm data/*

run:
	cargo run

format:
	cargo clippy
	cargo fmt

format_fix: format
	cargo clippy --fix --bin "webrs" --allow-dirty

test:
	ab -n 100000 -c 1000 http://localhost:8080/
	

.PHONY: all build clean run