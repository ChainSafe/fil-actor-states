#!/bin/bash
# Checks if all crates compile fine individually

set -euxo pipefail

function check_crate {
    pushd "$1" > /dev/null
    cargo check
    cargo test --no-run
    popd > /dev/null
}

for actor in actors/*; do
    if [ -d "$actor" ]; then
      check_crate "$actor"
    fi
done

others=(fil_actor_interface fil_actors_shared)
for other in "${others[@]}"; do
    check_crate "$other"
done

