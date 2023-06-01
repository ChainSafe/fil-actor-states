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
	sed -i -e 's|fil_actor_interface = "[0-9]\+"|fil_actor_interface = { path = "../fil_actor_interface" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actors_shared = "[0-9]\+"|fil_actors_shared = { path =  "../fil_actors_shared" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_account_state = "[0-9]\+"|fil_actor_account_state = { path =  "../actors/account" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_init_state = "[0-9]\+"|fil_actor_init_state = { path =  "../actors/init" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_miner_state = "[0-9]\+"|fil_actor_miner_state = { path =  "../actors/miner" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_power_state = "[0-9]\+"|fil_actor_power_state = { path =  "../actors/power" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_system_state = "[0-9]\+"|fil_actor_system_state = { path =  "../actors/system" }|g' ./forest/Cargo.toml

.PHONY: install-lint-tools lint-all audit udeps lint lint-clippy fmt clean update-forest
