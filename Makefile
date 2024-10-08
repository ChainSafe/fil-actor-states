install-lint-tools:
	cargo install --locked taplo-cli
	cargo install --locked cargo-audit
	cargo install --locked cargo-spellcheck
	cargo install --locked cargo-udeps

# Lints with everything we have in our CI arsenal
lint-all: lint audit udeps

check:
	bash check_crates.sh

audit:
	cargo audit

udeps:
	cargo +nightly udeps

lint: clean lint-clippy
	cargo fmt --all --check
	taplo fmt --check
	taplo lint
	
lint-clippy:
	cargo clippy --all-features --all-targets -- -D warnings

# Formats Rust and TOML files
fmt:
	cargo fmt --all
	taplo fmt

clean:
	@echo "Cleaning local packages..."
	@cargo clean
	@echo "Done cleaning."

update-forest:
	git submodule update --init --recursive --remote

modify-forest:
	# Set dependencies to this repository
	sed -i -e 's|fil_actor_interface =.*|fil_actor_interface = { path = "../fil_actor_interface" }|g' ./forest/Cargo.toml
	sed -i -E 's|fil_actors_shared = \{ version = "[^"]*", features = \[(.*)\] \}|fil_actors_shared = { path = "../fil_actors_shared", features = [\1] }|g' ./forest/Cargo.toml
	sed -i -E 's|(fil_actors_shared = \{).*git =.*branch =.*,(.*features =.*)|\1 path = "../fil_actors_shared",\2|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_account_state =.*|fil_actor_account_state = { path = "../actors/account" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_init_state =.*|fil_actor_init_state = { path = "../actors/init" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_miner_state =.*|fil_actor_miner_state = { path = "../actors/miner" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_power_state =.*|fil_actor_power_state = { path = "../actors/power" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_system_state =.*|fil_actor_system_state = { path = "../actors/system" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_datacap_state =.*|fil_actor_datacap_state = { path = "../actors/datacap" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_cron_state =.*|fil_actor_cron_state = { path = "../actors/cron" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_reward_state =.*|fil_actor_reward_state = { path = "../actors/reward" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_verifreg_state =.*|fil_actor_verifreg_state = { path = "../actors/verifreg" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_market_state =.*|fil_actor_market_state = { path = "../actors/market" }|g' ./forest/Cargo.toml

.PHONY: install-lint-tools lint-all audit udeps lint lint-clippy fmt clean update-forest
