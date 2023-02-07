
install-deps:
	apt-get update -y
	apt-get install --no-install-recommends -y build-essential clang protobuf-compiler ocl-icd-opencl-dev aria2 cmake

install-lint-tools:
	cargo install --locked taplo-cli
	cargo install --locked cargo-audit
	cargo install --locked cargo-spellcheck
	cargo install --locked cargo-udeps

clean-all:
	cargo clean

# Lints with everything we have in our CI arsenal
lint-all: lint audit spellcheck udeps

audit:
	cargo audit

udeps:
	cargo udeps

spellcheck:
	cargo spellcheck --code 1

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
	@cargo clean -p runtime_v8
  	@cargo clean -p runtime_v9
  	@cargo clean -p runtime_v10
  	@cargo clean -p account_v8
  	@cargo clean -p account_v9
  	@cargo clean -p account_v10
  	@cargo clean -p init_v8
  	@cargo clean -p init_v9
  	@cargo clean -p init_v10
  	@cargo clean -p cron_v8
  	@cargo clean -p cron_v9
  	@cargo clean -p cron_v10
  	@cargo clean -p paych_v8
  	@cargo clean -p system_v8
  	@cargo clean -p system_v9
  	@cargo clean -p system_v10
  	@cargo clean -p multisig_v8
  	@cargo clean -p multisig_v9
  	@cargo clean -p multisig_v10
  	@cargo clean -p market_v8
  	@cargo clean -p market_v9
  	@cargo clean -p power_v8
  	@cargo clean -p reward_v8
  	@cargo clean -p paych_v9
  	@cargo clean -p paych_v10
  	@cargo clean -p power_v9
  	@cargo clean -p power_v10
  	@cargo clean -p reward_v9
  	@cargo clean -p reward_v10
  	@cargo clean -p verifreg_v8
  	@cargo clean -p verifreg_v9
  	@cargo clean -p miner_v8
  	@cargo clean -p miner_v9
  	@cargo clean -p verifreg_v10
	@echo "Done cleaning."

.PHONY: clean clean-all lint lint-clippy install-lint-tools install-deps
