install-lint-tools:
	cargo install --locked taplo-cli
	cargo install --locked cargo-audit
	cargo install --locked cargo-spellcheck
	cargo install --locked cargo-udeps

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
	cargo clippy --all-features -- -D warnings

# Formats Rust and TOML files
fmt:
	cargo fmt --all
	taplo fmt

clean:
	@echo "Cleaning local packages..."
	@cargo clean
	@echo "Done cleaning."

update-forest:
	git submodule update --init --recursive

modify-forest:
	cargo add --replace fil_actor_interface --path ../fil_actor_interface
	cargo add --replace fil_actors_shared --path ../fil_actors_shared
	cargo add --replace fil_actor_account_state --path ../actors/account
	cargo add --replace fil_actor_init_state --path ../actors/init
	cargo add --replace fil_actor_miner_state --path ../actors/miner
	cargo add --replace fil_actor_power_state --path ../actors/power
	cargo add --replace fil_actor_system_state --path ../actors/system

.PHONY: install-lint-tools lint-all audit udeps lint lint-clippy fmt clean update-forest
