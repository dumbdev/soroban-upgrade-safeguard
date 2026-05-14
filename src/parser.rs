use anyhow::{Result, Context};
use wasmparser::{Parser, Payload};

/// Represents the extracted Soroban-specific custom sections from a WASM module.
#[derive(Debug, Default)]
pub struct SorobanMetadata {
    pub spec: Vec<Vec<u8>>,
    pub env_meta: Option<Vec<u8>>,
}

/// Parses the WASM bytes to extract Soroban-specific custom sections.
pub fn extract_metadata(bytes: &[u8]) -> Result<SorobanMetadata> {
    let mut metadata = SorobanMetadata::default();
    let parser = Parser::new(0);

    for payload in parser.parse_all(bytes) {
        match payload.context("Failed to parse WASM payload")? {
            Payload::CustomSection(section) => {
                match section.name() {
                    "contractspecv0" => {
                        metadata.spec.push(section.data().to_vec());
                    }
                    "contractenvmetav0" => {
                        metadata.env_meta = Some(section.data().to_vec());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(metadata)
}
