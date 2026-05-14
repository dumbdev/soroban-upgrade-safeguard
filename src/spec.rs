use std::collections::HashMap;

use stellar_xdr::curr::{
    ScSpecEntry, ScSpecFunctionV0, ScSpecUdtEnumV0, ScSpecUdtErrorEnumV0, ScSpecUdtStructV0,
    ScSpecUdtUnionV0,
};

/// A structured representation of a Soroban contract's public interface,
/// organized by type for easy comparison between contract versions.
#[derive(Debug, Default)]
pub struct ContractSpec {
    /// Contract functions, keyed by name.
    pub functions: HashMap<String, ScSpecFunctionV0>,
    /// User-defined structs, keyed by name.
    pub structs: HashMap<String, ScSpecUdtStructV0>,
    /// User-defined enums, keyed by name.
    pub enums: HashMap<String, ScSpecUdtEnumV0>,
    /// User-defined unions (tagged enums with data), keyed by name.
    pub unions: HashMap<String, ScSpecUdtUnionV0>,
    /// Error enums, keyed by name.
    pub error_enums: HashMap<String, ScSpecUdtErrorEnumV0>,
}

impl ContractSpec {
    /// Build a `ContractSpec` from a list of decoded `ScSpecEntry` objects.
    pub fn from_entries(entries: &[ScSpecEntry]) -> Self {
        let mut spec = ContractSpec::default();

        for entry in entries {
            match entry {
                ScSpecEntry::FunctionV0(f) => {
                    let name = f.name.to_string();
                    spec.functions.insert(name, f.clone());
                }
                ScSpecEntry::UdtStructV0(s) => {
                    let name = s.name.to_string();
                    spec.structs.insert(name, s.clone());
                }
                ScSpecEntry::UdtEnumV0(e) => {
                    let name = e.name.to_string();
                    spec.enums.insert(name, e.clone());
                }
                ScSpecEntry::UdtUnionV0(u) => {
                    let name = u.name.to_string();
                    spec.unions.insert(name, u.clone());
                }
                ScSpecEntry::UdtErrorEnumV0(e) => {
                    let name = e.name.to_string();
                    spec.error_enums.insert(name, e.clone());
                }
            }
        }

        spec
    }

    /// Returns a summary string of the spec contents.
    pub fn summary(&self) -> String {
        format!(
            "Functions: {}, Structs: {}, Enums: {}, Unions: {}, Errors: {}",
            self.functions.len(),
            self.structs.len(),
            self.enums.len(),
            self.unions.len(),
            self.error_enums.len(),
        )
    }
}
