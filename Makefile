install-lint-tools:
	cargo install --locked taplo-cli
	cargo install --locked cargo-audit
	cargo install --locked cargo-spellcheck
	cargo install --locked cargo-udeps

install-deps:
	apt-get update -y
	apt-get install --no-install-recommends -y protobuf-compiler

clean-all:
	cargo clean

# Lints with everything we have in our CI arsenal
lint-all: lint audit udeps

audit:
	cargo audit

udeps:
	cargo udeps

lint: clean lint-clippy
	cargo fmt --all --check
	taplo fmt --check
	taplo lint
	
lint-clippy:
	cargo clippy

# Formats Rust and TOML files
fmt:
	cargo fmt --all
	taplo fmt

clean:
	@echo "Cleaning local packages..."
	@cargo clean
	@echo "Done cleaning."

.PHONY: clean clean-all lint lint-clippy install-lint-tools
