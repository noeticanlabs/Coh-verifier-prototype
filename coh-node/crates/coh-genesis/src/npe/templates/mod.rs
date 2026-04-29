//! Coh Proof Template Registry
//!
//! Defines recurring theorem shapes and their associated tactic skeletons.

use serde::{Deserialize, Serialize};

/// Recurring Coh theorem patterns
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CohTemplateKind {
    /// V(z) + (s1 + s2) <= V(x) + (d1 + d2) + (a1 + a2)
    CertifiedComposition,
    /// V(x) + 0 <= V(x) + 0 + 0
    IdentityCertification,
    /// d_total <= d1 + d2
    EnvelopeDomination,
    /// df <= dg, dg <= dh => df <= dh
    HomPreorder,
    /// df1 <= df2, dg1 <= dg2 => df1 + dg1 <= df2 + dg2
    MonotoneComposition,
    /// d(x,z) <= d(x,y) + d(y,z)
    LawvereTriangle,
    /// Induction-based budget tracking over multiple steps
    BudgetTelescoping,
}

impl CohTemplateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CohTemplateKind::CertifiedComposition => "CertifiedComposition",
            CohTemplateKind::IdentityCertification => "IdentityCertification",
            CohTemplateKind::EnvelopeDomination => "EnvelopeDomination",
            CohTemplateKind::HomPreorder => "HomPreorder",
            CohTemplateKind::MonotoneComposition => "MonotoneComposition",
            CohTemplateKind::LawvereTriangle => "LawvereTriangle",
            CohTemplateKind::BudgetTelescoping => "BudgetTelescoping",
        }
    }

    /// Preferred tactic sequence for this template
    pub fn preferred_tactics(&self) -> Vec<&'static str> {
        match self {
            CohTemplateKind::CertifiedComposition => vec!["linarith", "nlinarith", "calc"],
            CohTemplateKind::IdentityCertification => vec!["simp", "rfl"],
            CohTemplateKind::EnvelopeDomination => vec!["linarith", "exact"],
            CohTemplateKind::HomPreorder => vec!["exact le_trans", "linarith"],
            CohTemplateKind::MonotoneComposition => vec!["exact add_le_add", "linarith"],
            CohTemplateKind::LawvereTriangle => vec!["by_cases", "linarith", "exact"],
            CohTemplateKind::BudgetTelescoping => vec!["induction", "simp", "linarith"],
        }
    }
}

/// Classify a Lean goal into a Coh template
pub fn classify_coh_template(goal: &str) -> Option<CohTemplateKind> {
    // Structural pattern matching (simplified heuristics)
    if goal.contains(" + ")
        && goal.contains(" ≤ ")
        && (goal.contains('x') || goal.contains('y') || goal.contains('z'))
        && (goal.contains("Spend") || goal.contains("Defect") || goal.contains("Authority"))
    {
        return Some(CohTemplateKind::CertifiedComposition);
    }

    if goal.contains(" + 0 ≤ ") || goal.contains(" + 0 + 0") {
        return Some(CohTemplateKind::IdentityCertification);
    }

    if goal.contains("delta") || (goal.contains('d') && goal.contains(" + ")) {
        return Some(CohTemplateKind::EnvelopeDomination);
    }

    if goal.contains("le_trans") || (goal.contains("<=") && goal.matches("<=").count() >= 2) {
        return Some(CohTemplateKind::HomPreorder);
    }

    if goal.contains("induction") || goal.contains("trace") || goal.contains("sum") {
        return Some(CohTemplateKind::BudgetTelescoping);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_composition() {
        let goal = "obj.V x3 + (obj.Spend R1 + obj.Spend R2) ≤ obj.V x1 + (obj.Defect R1 + obj.Defect R2) + (obj.Authority R1 + obj.Authority R2)";
        assert_eq!(classify_coh_template(goal), Some(CohTemplateKind::CertifiedComposition));
    }

    #[test]
    fn test_classify_identity() {
        let goal = "obj.V x + 0 ≤ obj.V x + 0 + 0";
        assert_eq!(classify_coh_template(goal), Some(CohTemplateKind::IdentityCertification));
    }
}
