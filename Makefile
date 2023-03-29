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
	sed -i -e 's|fil_actor_interface = { git = "https://github.com/ChainSafe/fil-actor-states" }|fil_actor_interface = { path = "../fil_actor_interface" }|g' -e 's|fil_actors_runtime_v10 = { git = "https://github.com/ChainSafe/fil-actor-states" }|fil_actors_runtime_v10 = { path =  "../runtime_v10" }|g' forest/Cargo.toml

.PHONY: install-lint-tools lint-all audit udeps lint lint-clippy fmt clean update-forest
