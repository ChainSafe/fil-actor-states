# Denotes the architecture of the machine. This is required for direct binary downloads.
# Note that some repositories might use different names for the same architecture.
CPU_ARCH := $(shell \
  ARCH=$$(uname -m); \
  if [ "$$ARCH" = "arm64" ]; then \
    ARCH="aarch64"; \
  fi; \
  echo "$$ARCH" \
)

install-cargo-binstall:
	wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-$(CPU_ARCH)-unknown-linux-musl.tgz
	tar xzf cargo-binstall-$(CPU_ARCH)-unknown-linux-musl.tgz
	cp cargo-binstall ~/.cargo/bin/cargo-binstall

install-lint-tools: install-cargo-binstall
	cargo binstall --no-confirm taplo-cli cargo-deny cargo-udeps

# Lints with everything we have in our CI arsenal
lint-all: lint deny udeps

check:
	bash check_crates.sh

check-with-validation:
	FIL_ENABLE_ACTOR_VERSION_CHECK=1 bash check_crates.sh

deny:
	cargo deny check bans licenses sources || (echo "See deny.toml"; false)

deny-advisories:
	cargo deny check advisories || (echo "See deny.toml"; false)

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
	sed -i -E 's|fil_actors_shared = \{ version = "[^"]*", features = \[(.*)\] \}|fil_actors_shared = { path = "../fil_actors_shared", features = [\1] }|g' ./forest/Cargo.toml
	sed -i -E 's|(fil_actors_shared = \{).*git =.*branch =.*,(.*features =.*)|\1 path = "../fil_actors_shared",\2|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_account_state =.*|fil_actor_account_state = { path = "../actors/account" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_init_state =.*|fil_actor_init_state = { path = "../actors/init" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_miner_state =.*|fil_actor_miner_state = { path = "../actors/miner" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_evm_state =.*|fil_actor_evm_state = { path = "../actors/evm" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_eam_state =.*|fil_actor_eam_state = { path = "../actors/eam" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_ethaccount_state =.*|fil_actor_ethaccount_state = { path = "../actors/ethaccount" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_multisig_state =.*|fil_actor_multisig_state = { path = "../actors/multisig" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_paych_state =.*|fil_actor_paych_state = { path = "../actors/paych" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_power_state =.*|fil_actor_power_state = { path = "../actors/power" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_system_state =.*|fil_actor_system_state = { path = "../actors/system" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_datacap_state =.*|fil_actor_datacap_state = { path = "../actors/datacap" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_cron_state =.*|fil_actor_cron_state = { path = "../actors/cron" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_reward_state =.*|fil_actor_reward_state = { path = "../actors/reward" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_verifreg_state =.*|fil_actor_verifreg_state = { path = "../actors/verifreg" }|g' ./forest/Cargo.toml
	sed -i -e 's|fil_actor_market_state =.*|fil_actor_market_state = { path = "../actors/market" }|g' ./forest/Cargo.toml

.PHONY: install-cargo-binstall install-lint-tools lint-all check deny deny-advisories udeps lint lint-clippy fmt clean update-forest modify-forest
