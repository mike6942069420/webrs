BIN=webrs
RUST_TARGET=x86_64-unknown-linux-musl

all: build

create_dirs:
	mkdir -p data
	touch data/log.txt

build:
	RUSTFLAGS="-C target-cpu=znver2" cargo build --target $(RUST_TARGET) --release

release_run: create_dirs build
	docker compose up --build

clean:
	cargo clean
	rm -rf data

run: create_dirs
	cargo run

format:
	cargo clippy
	cargo fmt

format_fix: format
	cargo clippy --fix --bin "webrs" --allow-dirty

test:
	ab -n 100000 -c 1000 http://localhost:8080/
	

.PHONY: all build clean run