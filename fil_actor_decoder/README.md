# fil_actor_decoder

Decode Filecoin built-in actor params and returns from raw CBOR to JSON.

Supports **datacap** (f07) and **verifreg** (f06) actors across all versions (v9-v17).

## CLI

```
fil-decode <COMMAND>

Commands:
  params  Decode method params
  return  Decode method return value
```

### Examples

Decode by explicit version:

```sh
fil-decode params \
  --actor verifreg --method 4 --version v17 \
  --hex 825501eb50a2528a325eadc4bb68d975a4d1700c9eeaa3480001900000000000
```

```json
{
  "address": "f15nikeuukgjpk3rf3ndmxljgroagj52vdgznkcwy",
  "allowance": "439804651110400"
}
```

Resolve version from network + epoch:

```sh
fil-decode params \
  --actor verifreg --method 4 \
  --network mainnet --epoch 5844742 \
  --hex 825501eb50a2528a325eadc4bb68d975a4d1700c9eeaa3480001900000000000
```

Decode params with a different actor and method:

```sh
fil-decode params \
  --actor verifreg --method 8 --version v17 \
  --hex 821a00303d2980
```

```json
{
  "allocation_ids": [],
  "client": 3161385
}
```

### Sourcing CBOR hex

From Filecoin JSON-RPC (`Filecoin.ChainGetMessage` for params, `Filecoin.StateReplay` for returns):

```sh
curl -s -X POST https://filfox.info/rpc/v1 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"Filecoin.ChainGetMessage","params":[{"/":"bafy2bzacebr2jvq..."}],"id":1}'
```

The `Params` field in the response is base64-encoded CBOR.

## Library API (unstable)

> **Note:** This is a POC. The library API is not stable and should not be depended on yet.
> Use the CLI for now; the Rust API will be finalized in a later iteration.

```rust
use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};

let json = decode_params(
    ActorType::VerifiedRegistry,
    ActorVersion::V17,
    4, // method number
    &cbor_bytes,
)?;
```

Network/epoch resolution:

```rust
use fil_actor_decoder::network::{Network, resolve_actor_version};

let version = resolve_actor_version(Network::Mainnet, 5_844_742)?;
```

## Version support

| Actor | Versions | Method numbers |
|-------|----------|---------------|
| datacap (f07) | v9: legacy (Mint=2..Allowance=21) | v10+: FRC-0042 hashes |
| verifreg (f06) | v9+: numeric + FRC-0042 | v12+: nested SectorAllocationClaims |

Key structural differences between versions:
- **datacap v9** has only MintParams/DestroyParams; v10 adds GranularityReturn; v12+ has full type set
- **verifreg v9** uses flat `SectorAllocationClaim` and `Address`-based providers
- **verifreg v10-v11** uses flat `SectorAllocationClaim` with `ActorID`-based providers
- **verifreg v12+** uses nested `SectorAllocationClaims` (CBOR-identical across v12-v17)

## Directory structure

```
fil_actor_decoder/
  src/
    actors/     # Per-actor decoders (datacap, verifreg) with version dispatch
    bin/        # CLI binary
  tests/
    fixtures/   # Large CBOR hex fixtures
    snapshots/  # Insta snapshot files
```

## Development

All tasks are wired through [mise](https://mise.jdx.dev/). Run from the workspace root (`fil-actor-states/`):

```sh
mise run decoder:test          # unit + snapshot tests
mise run decoder:test-all      # above + type-level JSON tests across all actor versions
mise run decoder:clippy        # lint decoder + shared crates
mise run decoder:snap-review   # review pending snapshot changes
mise run decoder:snap-accept   # accept all pending snapshot changes
```

Snapshot tests use both real on-chain CBOR (sourced via filfox RPC) and synthetic data constructed from actual Filecoin types.
