//! Semantic Layer - Mirroring Lean Axioms
//!
//! Implements the semantic cost and defect bound checks.
//! Defect Bound: delta(trace) <= defect

use crate::types::MicroReceipt;

/// Registry for semantic delta values (minimum costs)
pub struct SemanticRegistry;

impl SemanticRegistry {
    /// Get the minimum cost (delta) for a given step type.
    /// Based on Lean's `delta S trace`.
    pub fn delta_for_type(step_type: &Option<String>) -> u128 {
        match step_type {
            Some(t) => match t.as_str() {
                "coh.step.identity" => 0,
                "coh.step.transfer" => 5, // Example: minimum cost of a transfer
                "coh.step.mint" => 0,
                "coh.step.burn" => 0,
                _ => 0, // Default to 0 for unknown types (permissive)
            },
            None => 0,
        }
    }

    /// Check if the receipt's defect bound is satisfied: delta(trace) <= defect
    pub fn verify_defect_bound(receipt: &MicroReceipt) -> bool {
        let delta = Self::delta_for_type(&receipt.step_type);
        receipt.metrics.defect >= delta
    }

    /// Check if a step is an identity transition.
    /// Lean Axiom: Identity traces have zero cost.
    pub fn is_identity(step_type: &Option<String>) -> bool {
        matches!(step_type, Some(t) if t == "coh.step.identity")
    }
}
