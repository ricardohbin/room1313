# Room 13: convenience targets. Run `make help` for the list.

PORT ?= 8765
WASM_TARGET := wasm32-unknown-unknown
PKG_DIR := www/pkg
WASM_PACK := wasm-pack build --target web --out-dir $(PKG_DIR) --no-pack
BUILD_DIR := build
# Version shown in the corner of the game (read by src/lib.rs at compile time).
export ROOM_VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1).$(shell git rev-list --count HEAD)
# Renames www/pkg files with a content hash and renders www/index.html.
STAMP := python3 scripts/stamp.py
RELEASE_ZIP := $(BUILD_DIR)/room1313-build$(shell git rev-list --count main)

.DEFAULT_GOAL := help
.PHONY: help setup build release dev serve run test lint fmt check clean

help: ## Show this help
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  make %-8s %s\n", $$1, $$2}'

setup: ## Install the wasm target and wasm-pack (first time on a new machine)
	@command -v rustup >/dev/null || { echo "rustup not found: install it from https://rustup.rs"; exit 1; }
	rustup target add $(WASM_TARGET)
	@command -v wasm-pack >/dev/null || cargo install wasm-pack
	@command -v python3 >/dev/null || echo "warning: python3 not found; 'make serve' needs it"

build: ## Release build into www/pkg, with content-hashed file names
	$(WASM_PACK)
	$(STAMP)

release: build ## Release build, then zip www/ into build/room1313-build<commits on main>.zip
	@mkdir -p $(BUILD_DIR)
	@rm -f $(RELEASE_ZIP).zip
	python3 -c "import shutil; shutil.make_archive('$(RELEASE_ZIP)', 'zip', 'www')"
	@echo "Wrote $(RELEASE_ZIP).zip"

dev: ## Faster, unoptimized debug build into www/pkg
	$(WASM_PACK) --dev
	$(STAMP)

serve: ## Serve www/ on localhost (PORT=8765 by default)
	@echo "Room 13 at http://localhost:$(PORT) (Ctrl+C to stop)"
	python3 -m http.server $(PORT) -d www

run: build serve ## Build, then serve

test: ## Native unit tests
	cargo test

lint: ## Clippy for the wasm target
	cargo clippy --target $(WASM_TARGET)

fmt: ## Format the Rust code
	cargo fmt

check: lint test ## Lint and test

clean: ## Remove build output
	cargo clean
	rm -rf $(PKG_DIR) $(BUILD_DIR) www/index.html
