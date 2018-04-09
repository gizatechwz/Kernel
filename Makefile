# kernelkite — developer tasks
#
# The Rust workspace builds on any OS. The `/proc` backend only does real work
# on Linux; elsewhere use fixture replay (the `samples` target works anywhere).

CARGO ?= cargo
NPM   ?= npm
BIN   ?= target/release/kernelkite

.DEFAULT_GOAL := help

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN{FS=":.*?## "}{printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the Rust workspace (release)
	$(CARGO) build --release

.PHONY: test
test: ## Run all Rust tests
	$(CARGO) test --workspace

.PHONY: fmt
fmt: ## Format Rust sources
	$(CARGO) fmt --all
