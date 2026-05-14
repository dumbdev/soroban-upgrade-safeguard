use crate::diff::{DiffReport, Finding, Severity};
use std::collections::HashMap;

/// A structured container for aggregated comparison findings.
pub struct SafetyReport {
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub total_findings: usize,
    pub is_safe: bool,
    pub findings_by_category: HashMap<String, Vec<Finding>>,
}

impl SafetyReport {
    /// Compute a safety report from a raw DiffReport.
    pub fn new(diff: &DiffReport) -> Self {
        let mut critical_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;
        let mut findings_by_category: HashMap<String, Vec<Finding>> = HashMap::new();

        for finding in &diff.findings {
            match finding.severity {
                Severity::Critical => critical_count += 1,
                Severity::Warning => warning_count += 1,
                Severity::Info => info_count += 1,
            }
            findings_by_category
                .entry(finding.category.clone())
                .or_default()
                .push(finding.clone());
        }

        Self {
            critical_count,
            warning_count,
            info_count,
            total_findings: diff.findings.len(),
            is_safe: critical_count == 0,
            findings_by_category,
        }
    }

    /// Generate a structured, human-readable text output for the CLI.
    pub fn generate_summary_text(&self) -> String {
        let mut output = String::new();
        output.push_str("\n========================================\n");
        output.push_str("    SOROBAN UPGRADE SAFETY REPORT\n");
        output.push_str("========================================\n");
        
        let status = if self.is_safe { 
            "✅ PASSED (No breaking changes detected)" 
        } else { 
            "❌ FAILED (Critical breaking changes detected)" 
        };
        output.push_str(&format!("Status: {}\n", status));
        output.push_str(&format!("Critical: {}\n", self.critical_count));
        output.push_str(&format!("Warnings: {}\n", self.warning_count));
        output.push_str(&format!("Info:     {}\n", self.info_count));
        output.push_str("----------------------------------------\n\n");

        if self.total_findings == 0 {
            output.push_str("No relevant changes detected. The upgrade is identical in its exports and types.\n");
            return output;
        }

        // Sort categories to have consistent output
        let mut categories: Vec<&String> = self.findings_by_category.keys().collect();
        categories.sort();

        for category in categories {
            output.push_str(&format!("--- [{}] ---\n", category.to_ascii_uppercase()));
            let group = self.findings_by_category.get(category).unwrap();
            for finding in group {
                let icon = match finding.severity {
                    Severity::Critical => "🔴",
                    Severity::Warning => "🟡",
                    Severity::Info => "🔵",
                };
                output.push_str(&format!("{} {}\n", icon, finding.message));
            }
            output.push('\n');
        }

        if !self.is_safe {
            output.push_str("⚠️  ACTION REQUIRED: The new contract version modifies existing storage layouts or function interfaces.\n");
            output.push_str("Deploying this upgrade will result in orphaned data, serialization panics, or broken integrations.\n");
        }

        output
    }
}
