BIN=webrs
RUST_TARGET=x86_64-unknown-linux-musl

# Deployment vars
REMOTE_USER=mike
REMOTE_HOST=192.168.1.201
REMOTE_DIR=/home/mike/system-config/systems/server1/docker_containers
IMAGE_NAME=webrs:latest

# Get the version from Cargo.toml
VERSION := $(shell grep ^version Cargo.toml | head -1 | sed -E 's/version = "([^"]+)"/\1/')

all: build

create_dirs:
	mkdir -p data
	touch data/log.txt
	touch data/db.txt
	mkdir -p target/user_dir

build: create_dirs
	RUSTFLAGS="-C target-cpu=znver2" cargo build --target $(RUST_TARGET) --release

release_run: create_dirs build
	docker compose up --build

clean:
	cargo clean
	rm -rf data

run: create_dirs
	cargo run

format: build
	cargo fmt --all
	cargo clippy

format_fix: format
	cargo fmt -- --check
	cargo clippy --fix --bin "webrs" --allow-dirty

git: format_fix build
	@git add -A
	@git status
	@read -p "Commit message: " msg; \
	if [ -z "$$msg" ]; then \
		echo "Aborting commit: empty message"; exit 1; \
	fi; \
	git commit -m "$$msg"

	@git push origin main

	@if git rev-parse "$(VERSION)" >/dev/null 2>&1; then \
		echo "Tag $(VERSION) exists, skipping creation"; \
	else \
		git tag "$(VERSION)"; \
		git push origin "$(VERSION)"; \
		echo "Tag $(VERSION) created and pushed"; \
	fi 

deploy: format_fix build
	docker build -t $(IMAGE_NAME) .
	docker save $(IMAGE_NAME) -o webrs.tar
	scp webrs.tar $(REMOTE_USER)@$(REMOTE_HOST):/tmp/webrs.tar
	ssh -t $(REMOTE_USER)@$(REMOTE_HOST) "sudo docker load -i /tmp/webrs.tar && rm /tmp/webrs.tar"
	ssh -t $(REMOTE_USER)@$(REMOTE_HOST) "cd $(REMOTE_DIR) && sudo docker compose up -d --build"
	rm webrs.tar

make full: git deploy

.PHONY: all build clean run full