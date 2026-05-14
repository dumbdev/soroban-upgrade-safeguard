use crate::spec::ContractSpec;
use stellar_xdr::curr::{
    ScSpecFunctionInputV0, ScSpecFunctionV0, ScSpecTypeDef, ScSpecUdtEnumCaseV0, ScSpecUdtEnumV0,
    ScSpecUdtStructFieldV0, ScSpecUdtStructV0,
};

/// Severity of a detected issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

/// A single finding from the comparison analysis.
#[derive(Debug, Clone)]
pub struct Finding {
    pub severity: Severity,
    pub category: String,
    pub message: String,
}

/// Holds all findings from a comparison of two contract specs.
#[derive(Debug, Default)]
pub struct DiffReport {
    pub findings: Vec<Finding>,
}

impl DiffReport {
    pub fn critical_count(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Critical).count()
    }

    pub fn warning_count(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Warning).count()
    }

    pub fn info_count(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Info).count()
    }
}

/// Compare two contract specs and return a report of all findings.
pub fn compare(old: &ContractSpec, new: &ContractSpec) -> DiffReport {
    let mut report = DiffReport::default();

    compare_functions(old, new, &mut report);
    compare_structs(old, new, &mut report);
    compare_enums(old, new, &mut report);

    report
}

/// Compare function signatures between old and new contract specs.
fn compare_functions(old: &ContractSpec, new: &ContractSpec, report: &mut DiffReport) {
    // Check for removed or changed functions
    for (name, old_fn) in &old.functions {
        match new.functions.get(name) {
            None => {
                report.findings.push(Finding {
                    severity: Severity::Critical,
                    category: "Function Removed".to_string(),
                    message: format!(
                        "Function '{}' was removed. Existing callers will break.",
                        name
                    ),
                });
            }
            Some(new_fn) => {
                check_function_signature(name, old_fn, new_fn, report);
            }
        }
    }

    // Check for newly added functions (informational)
    for name in new.functions.keys() {
        if !old.functions.contains_key(name) {
            report.findings.push(Finding {
                severity: Severity::Info,
                category: "Function Added".to_string(),
                message: format!("New function '{}' added.", name),
            });
        }
    }
}

/// Compare signatures of two functions with the same name.
fn check_function_signature(
    name: &str,
    old_fn: &ScSpecFunctionV0,
    new_fn: &ScSpecFunctionV0,
    report: &mut DiffReport,
) {
    // Check input count
    let old_inputs: &[ScSpecFunctionInputV0] = old_fn.inputs.as_ref();
    let new_inputs: &[ScSpecFunctionInputV0] = new_fn.inputs.as_ref();

    if old_inputs.len() != new_inputs.len() {
        report.findings.push(Finding {
            severity: Severity::Critical,
            category: "Function Signature Changed".to_string(),
            message: format!(
                "Function '{}': parameter count changed from {} to {}.",
                name,
                old_inputs.len(),
                new_inputs.len()
            ),
        });
        return; // No point comparing individual params if count differs
    }

    // Check each input parameter
    for (i, (old_input, new_input)) in old_inputs.iter().zip(new_inputs.iter()).enumerate() {
        let old_name = old_input.name.to_string();
        let new_name = new_input.name.to_string();

        if old_name != new_name {
            report.findings.push(Finding {
                severity: Severity::Warning,
                category: "Parameter Renamed".to_string(),
                message: format!(
                    "Function '{}': parameter {} renamed from '{}' to '{}'.",
                    name, i, old_name, new_name
                ),
            });
        }

        if !types_equal(&old_input.type_, &new_input.type_) {
            report.findings.push(Finding {
                severity: Severity::Critical,
                category: "Parameter Type Changed".to_string(),
                message: format!(
                    "Function '{}': parameter {} ('{}') type changed from `{}` to `{}`.",
                    name, i, old_name, 
                    crate::mapper::type_to_string(&old_input.type_), 
                    crate::mapper::type_to_string(&new_input.type_)
                ),
            });
        }
    }

    // Check output types
    let old_outputs: &[ScSpecTypeDef] = old_fn.outputs.as_ref();
    let new_outputs: &[ScSpecTypeDef] = new_fn.outputs.as_ref();

    if old_outputs.len() != new_outputs.len() {
        report.findings.push(Finding {
            severity: Severity::Critical,
            category: "Return Type Changed".to_string(),
            message: format!(
                "Function '{}': return type count changed from {} to {}.",
                name,
                old_outputs.len(),
                new_outputs.len()
            ),
        });
    } else {
        for (i, (old_out, new_out)) in old_outputs.iter().zip(new_outputs.iter()).enumerate() {
            if !types_equal(old_out, new_out) {
                report.findings.push(Finding {
                    severity: Severity::Critical,
                    category: "Return Type Changed".to_string(),
                    message: format!(
                        "Function '{}': return type {} changed from `{}` to `{}`.",
                        name, i, 
                        crate::mapper::type_to_string(old_out), 
                        crate::mapper::type_to_string(new_out)
                    ),
                });
            }
        }
    }
}

/// Compare two ScSpecTypeDef values for equality.
/// We use the PartialEq derive on the XDR types.
fn types_equal(a: &ScSpecTypeDef, b: &ScSpecTypeDef) -> bool {
    a == b
}

/// Compare struct definitions between old and new contract specs.
fn compare_structs(old: &ContractSpec, new: &ContractSpec, report: &mut DiffReport) {
    for (name, old_struct) in &old.structs {
        match new.structs.get(name) {
            None => {
                report.findings.push(Finding {
                    severity: Severity::Critical,
                    category: "Struct Removed".to_string(),
                    message: format!(
                        "Struct '{}' was removed. Storage using this type will be unreadable.",
                        name
                    ),
                });
            }
            Some(new_struct) => {
                check_struct_fields(name, old_struct, new_struct, report);
            }
        }
    }

    // Check for newly added structs (informational)
    for name in new.structs.keys() {
        if !old.structs.contains_key(name) {
            report.findings.push(Finding {
                severity: Severity::Info,
                category: "Struct Added".to_string(),
                message: format!("New struct '{}' added.", name),
            });
        }
    }
}

/// Compare fields of two structs with the same name.
///
/// Soroban serializes struct fields by position order, so field reordering,
/// removal, or type changes all break storage layout compatibility.
fn check_struct_fields(
    name: &str,
    old_struct: &ScSpecUdtStructV0,
    new_struct: &ScSpecUdtStructV0,
    report: &mut DiffReport,
) {
    let old_fields: &[ScSpecUdtStructFieldV0] = old_struct.fields.as_ref();
    let new_fields: &[ScSpecUdtStructFieldV0] = new_struct.fields.as_ref();

    // Check for removed fields
    for old_field in old_fields {
        let old_name = old_field.name.to_string();
        let still_exists = new_fields.iter().any(|f| f.name.to_string() == old_name);
        if !still_exists {
            report.findings.push(Finding {
                severity: Severity::Critical,
                category: "Struct Field Removed".to_string(),
                message: format!(
                    "Struct '{}': field '{}' was removed. Existing storage will be corrupted.",
                    name, old_name
                ),
            });
        }
    }

    // Check fields that exist in both versions, by position
    for (i, (old_field, new_field)) in old_fields.iter().zip(new_fields.iter()).enumerate() {
        let old_name = old_field.name.to_string();
        let new_name = new_field.name.to_string();

        // Field at the same position has a different name — reordering detected
        if old_name != new_name {
            report.findings.push(Finding {
                severity: Severity::Critical,
                category: "Struct Field Reordered".to_string(),
                message: format!(
                    "Struct '{}': field at position {} changed from '{}' to '{}'. \
                     Positional serialization means this breaks storage layout.",
                    name, i, old_name, new_name
                ),
            });
        }

        // Field type changed
        if !types_equal(&old_field.type_, &new_field.type_) {
            report.findings.push(Finding {
                severity: Severity::Critical,
                category: "Struct Field Type Changed".to_string(),
                message: format!(
                    "Struct '{}': field '{}' (position {}) type changed from `{}` to `{}`.",
                    name, old_name, i, 
                    crate::mapper::type_to_string(&old_field.type_), 
                    crate::mapper::type_to_string(&new_field.type_)
                ),
            });
        }
    }

    // Check for new fields appended at the end
    if new_fields.len() > old_fields.len() {
        for new_field in &new_fields[old_fields.len()..] {
            report.findings.push(Finding {
                severity: Severity::Warning,
                category: "Struct Field Added".to_string(),
                message: format!(
                    "Struct '{}': new field '{}' appended. \
                     Existing storage entries won't have this field — ensure migration handles defaults.",
                    name,
                    new_field.name.to_string()
                ),
            });
        }
    }
}

/// Compare enum definitions between old and new contract specs.
fn compare_enums(old: &ContractSpec, new: &ContractSpec, report: &mut DiffReport) {
    for (name, old_enum) in &old.enums {
        match new.enums.get(name) {
            None => {
                report.findings.push(Finding {
                    severity: Severity::Critical,
                    category: "Enum Removed".to_string(),
                    message: format!(
                        "Enum '{}' was removed. Data using this type will be invalid.",
                        name
                    ),
                });
            }
            Some(new_enum) => {
                check_enum_cases(name, old_enum, new_enum, report);
            }
        }
    }

    // Check for newly added enums
    for name in new.enums.keys() {
        if !old.enums.contains_key(name) {
            report.findings.push(Finding {
                severity: Severity::Info,
                category: "Enum Added".to_string(),
                message: format!("New enum '{}' added.", name),
            });
        }
    }
}

/// Compare cases of two enums with the same name.
fn check_enum_cases(
    name: &str,
    old_enum: &ScSpecUdtEnumV0,
    new_enum: &ScSpecUdtEnumV0,
    report: &mut DiffReport,
) {
    let old_cases: &[ScSpecUdtEnumCaseV0] = old_enum.cases.as_ref();
    let new_cases: &[ScSpecUdtEnumCaseV0] = new_enum.cases.as_ref();

    for old_case in old_cases {
        let old_name = old_case.name.to_string();
        
        match new_cases.iter().find(|c| c.name.to_string() == old_name) {
            None => {
                // The case was removed entirely
                report.findings.push(Finding {
                    severity: Severity::Critical,
                    category: "Enum Case Removed".to_string(),
                    message: format!(
                        "Enum '{}': case '{}' (value: {}) was removed. \
                         On-chain data with this value will be orphaned/invalid.",
                        name, old_name, old_case.value
                    ),
                });
            }
            Some(new_case) => {
                // The case exists, but did its integer value change?
                if old_case.value != new_case.value {
                    report.findings.push(Finding {
                        severity: Severity::Critical,
                        category: "Enum Case Value Changed".to_string(),
                        message: format!(
                            "Enum '{}': case '{}' value changed from {} to {}. \
                             This breaks data serialization.",
                            name, old_name, old_case.value, new_case.value
                        ),
                    });
                }
            }
        }
    }

    // Check for new enum cases (usually safe, but good to know)
    if new_cases.len() > old_cases.len() {
        for new_case in new_cases {
            let new_name = new_case.name.to_string();
            if !old_cases.iter().any(|c| c.name.to_string() == new_name) {
                report.findings.push(Finding {
                    severity: Severity::Info,
                    category: "Enum Case Added".to_string(),
                    message: format!(
                        "Enum '{}': new case '{}' (value {}) added.",
                        name, new_name, new_case.value
                    ),
                });
            }
        }
    }
}
