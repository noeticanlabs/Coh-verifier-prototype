//! CTRL to CohBit Adapter
//!
//! This adapter bridges CTRL repair attempts to CohBit receipts,
//! making CTRL a CohBit-governed repair actor.
//!
//! The model:
//!   CTRL repair attempt = proposed state transition
//!   CohBit decides whether that transition is admissible.
//!
//! Full loop:
//!   TheoremFailure → CTRL diagnosis → RepairCandidate ranked → CohBit receipt generated
//!   → Accounting law check → PatchTransaction in temp → LeanWorker verifies
//!   → V3 receipt → sequence accumulator → CohBit emitted → audit trail

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp
fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn tagged_hash32(tag: &str, values: &[&str]) -> coh_core::types::Hash32 {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    for value in values {
        hasher.update(value.as_bytes());
    }
    coh_core::types::Hash32(hasher.finalize().into())
}

fn derived_prev_guard(receipt: &CtrlRepairReceipt, step_index: u64) -> coh_core::types::Hash32 {
    if step_index == 0 {
        coh_core::sequence_accumulator::GENESIS_GUARD
    } else {
        let step_index_str = step_index.to_string();
        tagged_hash32(
            "ctrl.prev_guard",
            &[
                receipt.sequence_accumulator.as_str(),
                receipt.theorem_hash_pre.as_str(),
                step_index_str.as_str(),
            ],
        )
    }
}

// Note: V3 wire conversion requires coh-core feature which is already a dependency
// The candidate_to_v3_wire function is available when coh-core is linked

/// CTRL repair receipt - bridges attempt to CohBit

/// CTRL repair receipt - bridges attempt to CohBit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CtrlRepairReceipt {
    /// Theorem name being repaired
    pub theorem_name: String,
    /// Theorem state hash before repair
    pub theorem_hash_pre: String,
    /// Theorem state hash after repair
    pub theorem_hash_post: String,
    /// Hash of the repair candidate
    pub candidate_hash: String,
    /// Hash of the tactic/lemma used
    pub tactic_hash: String,
    /// Hash of Lean verification result
    pub lean_result_hash: String,
    /// Hash of audit trail
    pub audit_hash: String,
    /// Spend: tactic cost, runtime, complexity
    pub spend: u128,
    /// Defect reserve: allowed proof uncertainty
    pub defect_reserve: u128,
    /// Authority: explicit permission budget
    pub authority: u128,
    /// Sequence accumulator for chain
    pub sequence_accumulator: String,
    /// Timestamp of repair
    pub timestamp: u64,
}

/// Accounting law interpretation for CTRL
/// v_post + spend ≤ v_pre + defect + authority
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CtrlAccountingBudget {
    /// v_pre: theorem repair budget before attempt
    pub pre_budget: u128,
    /// v_post: remaining proof confidence after attempt
    pub post_budget: u128,
    /// spend: tactic cost, runtime, file mutation cost
    pub spend: u128,
    /// defect: allowed proof uncertainty / known gap
    pub defect: u128,
    /// authority: explicit permission to spend more
    pub authority: u128,
}

impl CtrlAccountingBudget {
    /// Check if repair is admissible under accounting law
    pub fn is_admissible(&self) -> bool {
        self.post_budget + self.spend <= self.pre_budget + self.defect + self.authority
    }

    /// Calculate remaining margin
    pub fn margin(&self) -> i128 {
        (self.pre_budget + self.defect + self.authority) as i128
            - (self.post_budget + self.spend) as i128
    }

    /// Create initial budget for a repair attempt
    pub fn initial(complexity: u128, defect_budget: u128) -> Self {
        CtrlAccountingBudget {
            pre_budget: complexity,
            post_budget: complexity,
            spend: 0,
            defect: defect_budget,
            authority: 0,
        }
    }

    /// Record tactic cost
    pub fn with_tactic_cost(&mut self, cost: u128) {
        self.spend += cost;
    }

    /// Update post-repair confidence
    pub fn with_post_confidence(&mut self, confidence: u128) {
        self.post_budget = confidence;
    }
}

/// CTRL attempt result mapped to Coh objective
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CtrlObjectiveResult {
    /// Lean accepted the repair
    LeanAccepted {
        theorem_name: String,
        proof_hash: String,
        stdout_hash: String,
        stderr_hash: String,
    },
    /// Lean rejected the repair
    LeanRejected {
        theorem_name: String,
        error_kind: String,
        error_output: String,
    },
    /// Timeout or resource limit
    Timeout {
        theorem_name: String,
        elapsed_ms: u64,
    },
}

/// CTRL attempt mapped to CohBit candidate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CtrlCohBitCandidate {
    /// Repair receipt
    pub receipt: CtrlRepairReceipt,
    /// Objective result from Lean
    pub objective_result: CtrlObjectiveResult,
    /// Accounting budget check
    pub accounting_check: CtrlAccountingBudget,
    /// Whether this candidate is admissible
    pub admissible: bool,
}

impl CtrlCohBitCandidate {
    /// Create a candidate from CTRL attempt
    pub fn new(
        receipt: CtrlRepairReceipt,
        objective_result: CtrlObjectiveResult,
        accounting: CtrlAccountingBudget,
    ) -> Self {
        let admissible = matches!(objective_result, CtrlObjectiveResult::LeanAccepted { .. })
            && accounting.is_admissible();

        CtrlCohBitCandidate {
            receipt,
            objective_result,
            accounting_check: accounting,
            admissible,
        }
    }

    /// Get the decision for this candidate
    pub fn decision(&self) -> &'static str {
        if self.admissible {
            "Accept"
        } else {
            "Reject"
        }
    }
}

/// CTRL audit trail mapped to Coh trajectory
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CtrlCohTrajectory {
    /// All attempts in sequence
    pub attempts: Vec<TrajectoryEntry>,
    /// Sequence accumulator hash
    pub accumulator: String,
    /// Total successful repairs
    pub success_count: usize,
    /// Total failed attempts
    pub failure_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrajectoryEntry {
    /// Attempt index
    pub index: usize,
    /// Theorem name
    pub theorem_name: String,
    /// Candidate hash
    pub candidate_hash: String,
    /// Result
    pub result: String,
    /// Timestamp
    pub timestamp: u64,
}

impl CtrlCohTrajectory {
    /// Add an attempt to the trajectory
    pub fn add_attempt(
        &mut self,
        theorem_name: String,
        candidate_hash: String,
        result: &CtrlObjectiveResult,
    ) {
        let result_str = match result {
            CtrlObjectiveResult::LeanAccepted { .. } => "accepted".to_string(),
            CtrlObjectiveResult::LeanRejected { .. } => "rejected".to_string(),
            CtrlObjectiveResult::Timeout { .. } => "timeout".to_string(),
        };

        if result_str == "accepted" {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        self.attempts.push(TrajectoryEntry {
            index: self.attempts.len(),
            theorem_name,
            candidate_hash,
            result: result_str,
            timestamp: now_timestamp(),
        });

        self.accumulator = format!("acc_{}_{}", self.success_count, self.failure_count);
    }

    /// Check if trajectory is valid
    pub fn is_valid(&self, accounting: &CtrlAccountingBudget) -> bool {
        accounting.is_admissible() && self.success_count > 0
    }
}

/// Build CtrlRepairReceipt from CTRL components
#[allow(clippy::too_many_arguments)]
pub fn build_receipt(
    theorem_name: String,
    theorem_hash_pre: String,
    theorem_hash_post: String,
    candidate_hash: String,
    tactic_hash: String,
    lean_result_hash: String,
    audit_hash: String,
    spend: u128,
    defect_reserve: u128,
    authority: u128,
    sequence_accumulator: String,
) -> CtrlRepairReceipt {
    CtrlRepairReceipt {
        theorem_name,
        theorem_hash_pre,
        theorem_hash_post,
        candidate_hash,
        tactic_hash,
        lean_result_hash,
        audit_hash,
        spend,
        defect_reserve,
        authority,
        sequence_accumulator,
        timestamp: now_timestamp(),
    }
}

/// Convert CtrlCohBitCandidate to V3 wire for verification
/// This bridges CTRL to the real V3 verifier path
pub fn candidate_to_v3_wire(
    candidate: &CtrlCohBitCandidate,
    step_index: u64,
    step_type: Option<String>,
) -> Result<coh_core::types_v3::MicroReceiptV3Wire, String> {
    use coh_core::types::MetricsWire;
    use coh_core::types_v3::MicroReceiptV3Wire;
    use coh_core::types_v3::ObjectiveResult;

    // Map Lean result to ObjectiveResult
    let objective = match &candidate.objective_result {
        CtrlObjectiveResult::LeanAccepted {
            theorem_name,
            proof_hash,
            stdout_hash,
            stderr_hash,
        } => {
            let target = format!("{}:{}", theorem_name, proof_hash);
            ObjectiveResult::Satisfied(coh_core::types_v3::ObjectiveTarget::Custom(target))
        }
        CtrlObjectiveResult::LeanRejected {
            theorem_name,
            error_kind,
            error_output,
        } => {
            let target = format!("{}:{}:{}", theorem_name, error_kind, error_output);
            ObjectiveResult::Violated(coh_core::types_v3::ObjectiveTarget::Custom(target))
        }
        CtrlObjectiveResult::Timeout {
            theorem_name,
            elapsed_ms,
        } => {
            let target = format!("{}:timeout:{}ms", theorem_name, elapsed_ms);
            ObjectiveResult::Violated(coh_core::types_v3::ObjectiveTarget::Custom(target))
        }
    };

    let metrics = MetricsWire {
        v_pre: candidate.accounting_check.pre_budget.to_string(),
        v_post: candidate.accounting_check.post_budget.to_string(),
        spend: candidate.accounting_check.spend.to_string(),
        defect: candidate.accounting_check.defect.to_string(),
        authority: candidate.accounting_check.authority.to_string(),
        ..Default::default()
    };

    let step_index_str = step_index.to_string();
    let state_hash_prev = tagged_hash32(
        "ctrl.state.pre",
        &[
            candidate.receipt.theorem_hash_pre.as_str(),
            candidate.receipt.theorem_name.as_str(),
        ],
    );
    let state_hash_next = tagged_hash32(
        "ctrl.state.post",
        &[
            candidate.receipt.theorem_hash_post.as_str(),
            candidate.receipt.lean_result_hash.as_str(),
            candidate.receipt.theorem_name.as_str(),
        ],
    );
    let prev_guard = derived_prev_guard(&candidate.receipt, step_index);
    let chain_digest_next = tagged_hash32(
        "ctrl.chain.next",
        &[
            candidate.receipt.candidate_hash.as_str(),
            candidate.receipt.tactic_hash.as_str(),
            step_index_str.as_str(),
        ],
    );

    let mut wire = MicroReceiptV3Wire {
        schema_id: "coh.receipt.micro.v3".to_string(),
        version: "1.0.0".to_string(),
        object_id: candidate.receipt.candidate_hash.clone(),
        canon_profile_hash: tagged_hash32("ctrl.canon_profile", &["ctrl_repair_v1"]).to_hex(),
        policy_hash: tagged_hash32("ctrl.policy", &["ctrl_cohbit_v1"]).to_hex(),
        step_index,
        step_type,
        signatures: None,
        state_hash_prev: state_hash_prev.to_hex(),
        state_hash_next: state_hash_next.to_hex(),
        chain_digest_prev: prev_guard.to_hex(),
        chain_digest_next: chain_digest_next.to_hex(),
        metrics,
        objective_result: Some(objective),
        sequence_accumulator: None,
        override_applied: false,
    };

    let receipt_digest = coh_core::types_v3::compute_v3_canonical_digest(&wire)
        .map_err(|code| format!("failed to compute V3 canonical digest: {:?}", code))?;
    wire.sequence_accumulator = Some(
        coh_core::sequence_accumulator::compute_sequence_accumulator(
            prev_guard,
            receipt_digest,
            step_index,
            state_hash_prev,
            state_hash_next,
        ),
    );

    Ok(wire)
}

/// Compute V3-compatible digest for a CTRL repair receipt
/// This binds all fields - any change produces a different digest
pub fn compute_receipt_digest(receipt: &CtrlRepairReceipt) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash all binding fields - order matters
    receipt.theorem_hash_pre.hash(&mut hasher);
    receipt.theorem_hash_post.hash(&mut hasher);
    receipt.candidate_hash.hash(&mut hasher);
    receipt.tactic_hash.hash(&mut hasher);
    receipt.lean_result_hash.hash(&mut hasher);
    receipt.audit_hash.hash(&mut hasher);
    receipt.spend.hash(&mut hasher);
    receipt.defect_reserve.hash(&mut hasher);
    receipt.authority.hash(&mut hasher);
    receipt.sequence_accumulator.hash(&mut hasher);

    format!("{:016x}", hasher.finish())
}

/// Verify receipt integrity - returns true if digest matches
pub fn verify_receipt_integrity(receipt: &CtrlRepairReceipt, expected_digest: &str) -> bool {
    let computed = compute_receipt_digest(receipt);
    computed == expected_digest
}

/// Tamper detection - simulates tampering and returns new digest
pub fn tamper_and_redigest(
    receipt: &mut CtrlRepairReceipt,
    field: &str,
    new_value: &str,
) -> String {
    match field {
        "theorem_hash_post" => receipt.theorem_hash_post = new_value.to_string(),
        "tactic_hash" => receipt.tactic_hash = new_value.to_string(),
        "candidate_hash" => receipt.candidate_hash = new_value.to_string(),
        "sequence_accumulator" => receipt.sequence_accumulator = new_value.to_string(),
        _ => {}
    }
    compute_receipt_digest(receipt)
}

/// Convert CTRL attempt to CohBit receipt
/// Only successful attempts become CohBits
pub fn attempt_to_cohbit(
    attempt_result: &CtrlObjectiveResult,
    receipt: &CtrlRepairReceipt,
    accounting: &CtrlAccountingBudget,
) -> Option<CtrlCohBitCandidate> {
    match attempt_result {
        CtrlObjectiveResult::LeanAccepted { .. } => Some(CtrlCohBitCandidate::new(
            receipt.clone(),
            attempt_result.clone(),
            accounting.clone(),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accounting_admissible() {
        let mut budget = CtrlAccountingBudget::initial(1000, 100);
        budget.with_tactic_cost(50);
        budget.with_post_confidence(900);

        assert!(budget.is_admissible());
    }

    #[test]
    fn test_accounting_inadmissible() {
        let mut budget = CtrlAccountingBudget::initial(1000, 50);
        budget.with_tactic_cost(200);
        budget.with_post_confidence(900);

        assert!(!budget.is_admissible());
    }

    #[test]
    fn test_candidate_admissible() {
        let receipt = build_receipt(
            "theorem_1".to_string(),
            "hash_pre".to_string(),
            "hash_post".to_string(),
            "candidate".to_string(),
            "tactic".to_string(),
            "lean".to_string(),
            "audit".to_string(),
            50,
            100,
            0,
            "acc".to_string(),
        );

        let result = CtrlObjectiveResult::LeanAccepted {
            theorem_name: "theorem_1".to_string(),
            proof_hash: "proof".to_string(),
            stdout_hash: "out".to_string(),
            stderr_hash: "err".to_string(),
        };

        let mut accounting = CtrlAccountingBudget::initial(1000, 100);
        accounting.with_tactic_cost(50);
        accounting.with_post_confidence(900);

        let candidate = attempt_to_cohbit(&result, &receipt, &accounting);

        assert!(candidate.is_some());
        assert_eq!(candidate.unwrap().decision(), "Accept");
    }
}

/// Convert CtrlRepairReceipt to V3 wire format for chain verification
/// This enables CohBit candidates to be verified by coh-core's verify_micro_v3
pub fn receipt_to_v3_wire(
    receipt: &CtrlRepairReceipt,
    accounting: &CtrlAccountingBudget,
    step_index: u64,
) -> Result<coh_core::types_v3::MicroReceiptV3Wire, String> {
    use coh_core::types::MetricsWire;
    use coh_core::types_v3::MicroReceiptV3Wire;

    let metrics = MetricsWire {
        v_pre: accounting.pre_budget.to_string(),
        v_post: accounting.post_budget.to_string(),
        spend: accounting.spend.to_string(),
        defect: accounting.defect.to_string(),
        authority: accounting.authority.to_string(),
        ..Default::default()
    };

    let step_index_str = step_index.to_string();
    let state_hash_prev = tagged_hash32(
        "ctrl.state.pre",
        &[
            receipt.theorem_hash_pre.as_str(),
            receipt.theorem_name.as_str(),
        ],
    );
    let state_hash_next = tagged_hash32(
        "ctrl.state.post",
        &[
            receipt.theorem_hash_post.as_str(),
            receipt.lean_result_hash.as_str(),
            receipt.theorem_name.as_str(),
        ],
    );
    let prev_guard = derived_prev_guard(receipt, step_index);
    let chain_digest_next = tagged_hash32(
        "ctrl.chain.next",
        &[
            receipt.candidate_hash.as_str(),
            receipt.tactic_hash.as_str(),
            step_index_str.as_str(),
        ],
    );

    let mut wire = MicroReceiptV3Wire {
        schema_id: "coh.receipt.micro.v3".to_string(),
        version: "1.0.0".to_string(),
        object_id: receipt.candidate_hash.clone(),
        canon_profile_hash: tagged_hash32("ctrl.canon_profile", &["ctrl_repair_v1"]).to_hex(),
        policy_hash: tagged_hash32("ctrl.policy", &["ctrl_cohbit_v1"]).to_hex(),
        step_index,
        step_type: Some("CTRL_REPAIR".to_string()),
        signatures: None,
        state_hash_prev: state_hash_prev.to_hex(),
        state_hash_next: state_hash_next.to_hex(),
        chain_digest_prev: prev_guard.to_hex(),
        chain_digest_next: chain_digest_next.to_hex(),
        metrics,
        objective_result: Some(coh_core::types_v3::ObjectiveResult::Satisfied(
            coh_core::types_v3::ObjectiveTarget::Custom(receipt.theorem_name.clone()),
        )),
        sequence_accumulator: None,
        override_applied: false,
    };

    let receipt_digest = coh_core::types_v3::compute_v3_canonical_digest(&wire)
        .map_err(|code| format!("failed to compute V3 canonical digest: {:?}", code))?;
    wire.sequence_accumulator = Some(
        coh_core::sequence_accumulator::compute_sequence_accumulator(
            prev_guard,
            receipt_digest,
            step_index,
            state_hash_prev,
            state_hash_next,
        ),
    );

    Ok(wire)
}

/// Full verification: Convert CTRL receipt to V3 wire and verify with coh-core
/// Returns the verification result from coh-core's verify_micro_v3
pub fn verify_receipt_full(
    receipt: &CtrlRepairReceipt,
    accounting: &CtrlAccountingBudget,
    step_index: u64,
) -> coh_core::verify_micro_v3::VerifyMicroV3Result {
    use coh_core::types_v3::{PolicyGovernance, SequenceGuard, TieredConfig};
    use coh_core::verify_micro_v3::verify_micro_v3;

    let wire = match receipt_to_v3_wire(receipt, accounting, step_index) {
        Ok(wire) => wire,
        Err(message) => {
            return coh_core::verify_micro_v3::VerifyMicroV3Result {
                decision: coh_core::types::Decision::Reject,
                code: Some(coh_core::types::RejectCode::RejectSchema),
                message,
                step_index: Some(step_index),
                object_id: Some(receipt.candidate_hash.clone()),
                objective_checked: None,
                sequence_checked: Some(false),
                override_applied: Some(false),
            };
        }
    };
    let config = TieredConfig::default();
    let seq_guard = SequenceGuard::default();
    let policy_gov = PolicyGovernance::default();
    let prev_guard = derived_prev_guard(receipt, step_index);

    verify_micro_v3(
        wire,
        &config,
        &seq_guard,
        &policy_gov,
        None,
        Some(prev_guard),
        &coh_core::auth::VerifierContext::default(),
    )
}
