use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod loader;
mod parser;

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
    println!(
        "  ✅ Old: {} ({} bytes, {} spec entries)",
        old.path,
        old.bytes.len(),
        old_meta.spec.len()
    );

    // New WASM
    let new = loader::load_wasm(&args.new_wasm)?;
    let new_meta = parser::extract_metadata(&new.bytes)?;
    println!(
        "  ✅ New: {} ({} bytes, {} spec entries)",
        new.path,
        new.bytes.len(),
        new_meta.spec.len()
    );

    println!("\n✅ Metadata extracted successfully.");
    println!("   Next: Decoding XDR spec entries...");

    Ok(())
}


