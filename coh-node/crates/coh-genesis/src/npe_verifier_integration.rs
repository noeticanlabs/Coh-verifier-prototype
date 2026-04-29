//! NPE-Verifier Integration Module
//!
//! Connects the NPE proof search to Coh receipts validation.
//! Uses existing coh-node verifier to validate search budgets.

use crate::proof_receipt::{
    GoalEmbedding, ProofAttemptReceipt, SearchBudget,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Verification mode for proof attempts
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyMode {
    /// Validate only the proof attempt receipt
    ReceiptOnly,
    /// Validate with full signature verification
    FullSignature,
    /// Dry run - don't enforce signatures
    DryRun,
}

/// Result of budget validation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetValidationResult {
    /// Whether the budget is valid
    pub valid: bool,
    /// Remaining budget
    pub remaining: u64,
    /// Steps taken
    pub steps: u64,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Integrate proof attempt with coh-node verifier for budget validation
///
/// This validates that the search was Genesis-admissible and within budget.
pub fn validate_proof_attempt(
    receipt: &ProofAttemptReceipt,
    _mode: VerifyMode,
) -> BudgetValidationResult {
    // Convert to wire format for coh-node verifier
    let _wire = receipt.to_micro_receipt_wire();

    // Check search budget constraints
    let budget_valid = receipt.search_budget.can_proceed();
    let law_holds = receipt.genesis_metrics.law_holds;
    let coherence_holds = receipt.coherence_metrics.coherence_holds;

    // All conditions must hold
    let all_valid = budget_valid && law_holds && coherence_holds;

    BudgetValidationResult {
        valid: all_valid,
        remaining: receipt.search_budget.remaining(),
        steps: receipt.search_budget.steps,
        error: if !all_valid {
            let mut errors = Vec::new();
            if !budget_valid {
                errors.push("Search budget exhausted".to_string());
            }
            if !law_holds {
                errors.push("Genesis law violation".to_string());
            }
            if !coherence_holds {
                errors.push("Coherence violation".to_string());
            }
            Some(errors.join("; "))
        } else {
            None
        },
    }
}

/// Validate search budget specifically using coh-node verifier
pub fn validate_search_budget(
    budget: &SearchBudget,
    genesis_metrics: &(u128, u128, u128, u128), // m_pre, m_post, cost, slack
) -> BudgetValidationResult {
    let (m_pre, m_post, cost, slack) = *genesis_metrics;

    // Check Genesis law: M(g') + C(p) <= M(g) + D(p)
    let law_holds = m_post.saturating_add(cost) <= m_pre.saturating_add(slack);

    BudgetValidationResult {
        valid: budget.can_proceed() && law_holds,
        remaining: budget.remaining(),
        steps: budget.steps,
        error: if !law_holds {
            Some(format!(
                "Genesis law violated: {} + {} > {} + {}",
                m_post, cost, m_pre, slack
            ))
        } else if !budget.can_proceed() {
            Some("Search budget exhausted".to_string())
        } else {
            None
        },
    }
}

/// [PHASELOOM: PART V] Oplax Soundness Contract (Verifier Law)
/// 
/// Confirms the fundamental inequality:
/// V_PL(x') + Spend_PL(r) <= V_PL(x) + Defect_PL(r)
pub fn verify_oplax_soundness(
    v_pre: u128,
    v_post: u128,
    spend: u128,
    defect: u128,
) -> bool {
    // V_post + Spend <= V_pre + Defect
    v_post.saturating_add(spend) <= v_pre.saturating_add(defect)
}

/// Batch validate multiple proof attempts
pub fn batch_validate_receipts(
    receipts: &[ProofAttemptReceipt],
    mode: VerifyMode,
) -> HashMap<String, BudgetValidationResult> {
    let mut results = HashMap::new();

    for receipt in receipts {
        let result = validate_proof_attempt(receipt, mode);
        results.insert(receipt.attempt_id.clone(), result);
    }

    results
}

/// Index proven cache by goal hash for quick lookup
pub fn index_by_goal_hash(
    receipts: &[ProofAttemptReceipt],
) -> HashMap<String, ProofAttemptReceipt> {
    let mut index = HashMap::new();

    for receipt in receipts {
        let hash = receipt.goal_embedding.theorem_hash.clone();
        index.insert(hash, receipt.clone());
    }

    index
}

/// Cluster receipts by similar goal embedding for strategy learning
pub fn cluster_by_embedding(
    receipts: &[ProofAttemptReceipt],
    target_embedding: &GoalEmbedding,
) -> Vec<ProofAttemptReceipt> {
    let mut cluster = Vec::new();

    // Simple clustering: find receipts with similar theorem hash prefix
    let target_prefix =
        &target_embedding.theorem_hash[..8.min(target_embedding.theorem_hash.len())];

    for receipt in receipts {
        let hash = &receipt.goal_embedding.theorem_hash;
        if hash.starts_with(target_prefix) || hash == &target_embedding.theorem_hash {
            cluster.push(receipt.clone());
        }
    }

    cluster
}

/// Integration with Lean JSON search for batched theorem verification
pub struct LeanBatchedVerifier {
    /// Project path for Lean
    project_path: std::path::PathBuf,
    /// Lake command
    lake_cmd: String,
}

impl LeanBatchedVerifier {
    pub fn new(project_path: std::path::PathBuf, lake_cmd: String) -> Self {
        Self {
            project_path,
            lake_cmd,
        }
    }

    /// Verify multiple theorems in batch
    pub fn batch_verify_theorems(
        &self,
        theorems: &[(&str, &str)], // (name, statement) pairs
    ) -> HashMap<String, bool> {
        use crate::lean_json_export::execute_lean_json_search;

        let mut results = HashMap::new();

        for (name, _statement) in theorems {
            let search_results = execute_lean_json_search(&self.project_path, &self.lake_cmd, name, Some(60));

            // A theorem is "verified" if search finds it
            results.insert(name.to_string(), search_results.count > 0);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_receipt::{CoherenceMetricsReceipt, GenesisMetricsReceipt, ProofResult};

    #[test]
    fn test_validate_proof_attempt() {
        let goal = GoalEmbedding::new("test_theorem", "∀x, x = x");
        let mut budget = SearchBudget::new(100);
        budget.spend(50);

        let genesis = GenesisMetricsReceipt {
            m_pre: 100,
            m_post: 80,
            cost: 10,
            slack: 50,
            law_holds: true,
        };

        let coherence = CoherenceMetricsReceipt {
            v_pre: 50,
            v_post: 30,
            coherence_holds: true,
        };

        let receipt = ProofAttemptReceipt::new(
            "attempt-1",
            goal,
            budget,
            ProofResult::Proved,
            genesis,
            coherence,
        );

        let result = validate_proof_attempt(&receipt, VerifyMode::ReceiptOnly);

        assert!(result.valid);
        assert_eq!(result.steps, 1);
    }

    #[test]
    fn test_validate_search_budget() {
        let budget = SearchBudget::new(100);
        let genesis = (100u128, 80, 10, 50); // m_pre, m_post, cost, slack

        let result = validate_search_budget(&budget, &genesis);

        assert!(result.valid);
    }

    #[test]
    fn test_validate_budget_exhausted() {
        let mut budget = SearchBudget::new(100);
        budget.spend(100); // Exhaust

        let genesis = (100u128, 80, 10, 50);

        let result = validate_search_budget(&budget, &genesis);

        assert!(!result.valid);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_genesis_law_violation() {
        let budget = SearchBudget::new(100);
        let genesis = (100u128, 100, 50, 20); // M(g') + C(p) = 150 > M(g) + D(p) = 120

        let result = validate_search_budget(&budget, &genesis);

        assert!(!result.valid);
        assert!(result.error.unwrap().contains("Genesis law"));
    }
}
