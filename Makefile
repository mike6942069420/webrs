BIN=webrs
RUST_TARGET=x86_64-unknown-linux-musl

# CD vars
REMOTE_USER=mike
REMOTE_HOST=192.168.1.201
REMOTE_DIR=/home/mike/system-config/systems/server1/docker_containers
IMAGE_NAME=webrs:latest


all: build

create_dirs:
	mkdir -p data
	touch data/log.txt
	touch data/db.txt

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

deploy: format_fix build
	docker build -t $(IMAGE_NAME) .
	docker save $(IMAGE_NAME) -o webrs.tar
	scp webrs.tar $(REMOTE_USER)@$(REMOTE_HOST):/tmp/webrs.tar
	ssh -t $(REMOTE_USER)@$(REMOTE_HOST) "sudo docker load -i /tmp/webrs.tar && rm /tmp/webrs.tar"
	ssh -t $(REMOTE_USER)@$(REMOTE_HOST) "cd $(REMOTE_DIR) && sudo docker compose up -d --build"
	rm webrs.tar


.PHONY: all build clean run