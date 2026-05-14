use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod diff;
mod loader;
mod mapper;
mod parser;
mod report;
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

    // Run comparison
    println!("\n🔬 Analyzing changes...");
    let diff_report = diff::compare(&old_spec, &new_spec);

    // Generate Safety Report
    let safety_report = report::SafetyReport::new(&diff_report);
    println!("{}", safety_report.generate_summary_text());

    if !safety_report.is_safe {
        std::process::exit(1);
    }

    Ok(())
}

