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
	# Set GIT_LFS_SKIP_SMUDGE=1 explicitly to not waste git lfs bandwidth
	GIT_LFS_SKIP_SMUDGE=1 git submodule update --init --recursive

modify-forest:
	# Keep forest separate from the local workspace
	echo "[workspace]" >> ./forest/Cargo.toml

	# Get rid of patch overrides
	sed -i -e 's|fil_actor[a-z_]* = { git .*||g' ./forest/Cargo.toml

	# Set dependencies to this repository
	sed -i -e 's|fil_actor_interface = "[0-9]\+"|fil_actor_interface = { path = "../fil_actor_interface" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actors_shared = "[0-9]\+"|fil_actors_shared = { path =  "../fil_actors_shared" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_account_state = "[0-9]\+"|fil_actor_account_state = { path =  "../actors/account" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_init_state = "[0-9]\+"|fil_actor_init_state = { path =  "../actors/init" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_miner_state = "[0-9]\+"|fil_actor_miner_state = { path =  "../actors/miner" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_power_state = "[0-9]\+"|fil_actor_power_state = { path =  "../actors/power" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_system_state = "[0-9]\+"|fil_actor_system_state = { path =  "../actors/system" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_datacap_state = "[0-9]\+"|fil_actor_datacap_state = { path =  "../actors/datacap" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_cron_state = "[0-9]\+"|fil_actor_cron_state = { path =  "../actors/cron" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_reward_state = "[0-9]\+"|fil_actor_reward_state = { path =  "../actors/reward" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_verifreg_state = "[0-9]\+"|fil_actor_verifreg_state = { path =  "../actors/verifreg" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_market_state = "[0-9]\+"|fil_actor_market_state = { path =  "../actors/market" }|g' ./forest/Cargo.toml

.PHONY: install-lint-tools lint-all audit udeps lint lint-clippy fmt clean update-forest
