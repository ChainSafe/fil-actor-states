//! CLI tool for decoding Filecoin actor params/returns from CBOR to JSON.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "fil-decode", about = "Decode Filecoin actor CBOR params/returns to JSON")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Clone)]
struct DecodeArgs {
    /// Actor type: datacap, verifreg
    #[arg(long)]
    actor: String,
    /// Method number
    #[arg(long)]
    method: u64,
    /// Hex-encoded CBOR bytes (no 0x prefix)
    #[arg(long)]
    hex: String,
    /// Actor version (v16 or v17). If omitted, uses --network + --epoch.
    #[arg(long)]
    version: Option<String>,
    /// Network: mainnet or calibnet (used with --epoch)
    #[arg(long)]
    network: Option<String>,
    /// Epoch (used with --network)
    #[arg(long)]
    epoch: Option<i64>,
}

#[derive(Subcommand)]
enum Command {
    /// Decode method params
    Params(DecodeArgs),
    /// Decode method return value
    Return(DecodeArgs),
}

fn resolve_version(args: &DecodeArgs) -> Result<fil_actor_decoder::ActorVersion> {
    if let Some(v) = &args.version {
        return v.parse();
    }
    match (&args.network, args.epoch) {
        (Some(net), Some(ep)) => {
            let network: fil_actor_decoder::network::Network = net.parse()?;
            fil_actor_decoder::network::resolve_actor_version(network, ep)
        }
        _ => anyhow::bail!("Provide either --version or both --network and --epoch"),
    }
}

fn decode_hex(hex_str: &str) -> Result<Vec<u8>> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(clean).context("Invalid hex input")
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (args, decode_fn): (&DecodeArgs, fn(_, _, _, &[u8]) -> _) = match &cli.command {
        Command::Params(a) => (a, fil_actor_decoder::decode_params),
        Command::Return(a) => (a, fil_actor_decoder::decode_return),
    };

    let actor_type: fil_actor_decoder::ActorType = args.actor.parse()?;
    let ver = resolve_version(args)?;
    let bytes = decode_hex(&args.hex)?;
    let json = decode_fn(actor_type, ver, args.method, &bytes)?;

    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}
