# Room 13: convenience targets. Run `make help` for the list.

PORT ?= 8765
WASM_TARGET := wasm32-unknown-unknown
PKG_DIR := www/pkg
WASM_PACK := wasm-pack build --target web --out-dir $(PKG_DIR) --no-pack

.DEFAULT_GOAL := help
.PHONY: help setup build dev serve run test lint fmt check clean

help: ## Show this help
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  make %-8s %s\n", $$1, $$2}'

setup: ## Install the wasm target and wasm-pack (first time on a new machine)
	@command -v rustup >/dev/null || { echo "rustup not found: install it from https://rustup.rs"; exit 1; }
	rustup target add $(WASM_TARGET)
	@command -v wasm-pack >/dev/null || cargo install wasm-pack
	@command -v python3 >/dev/null || echo "warning: python3 not found; 'make serve' needs it"

build: ## Release build of the game into www/pkg
	$(WASM_PACK)

dev: ## Faster, unoptimized debug build into www/pkg
	$(WASM_PACK) --dev

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
	rm -rf $(PKG_DIR)
