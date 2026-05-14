use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod loader;
mod parser;
mod spec;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the previous (on-chain) WASM contract
    #[arg(value_name = "OLD_WASM")]
    old_wasm: PathBuf,

    /// Path to the new (to be deployed) WASM contract
    #[arg(value_name = "NEW_WASM")]
    new_wasm: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔍 Soroban Upgrade Safeguard");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("\n📦 Loading and Parsing contracts...");

    // Old WASM
    let old = loader::load_wasm(&args.old_wasm)?;
    let old_meta = parser::extract_metadata(&old.bytes)?;
    let old_spec = spec::ContractSpec::from_entries(&old_meta.spec);
    println!("  ✅ Old: {} ({} bytes)", old.path, old.bytes.len());
    println!("     └─ {}", old_spec.summary());

    // New WASM
    let new = loader::load_wasm(&args.new_wasm)?;
    let new_meta = parser::extract_metadata(&new.bytes)?;
    let new_spec = spec::ContractSpec::from_entries(&new_meta.spec);
    println!("  ✅ New: {} ({} bytes)", new.path, new.bytes.len());
    println!("     └─ {}", new_spec.summary());

    println!("\n✅ Contract specs decoded successfully.");
    println!("   Next: Comparing function signatures and types...");

    Ok(())
}
