use crate::repair_candidate::{RepairSource, RepairStrategy};
use serde::{Deserialize, Serialize};

/// Result of an individual repair attempt
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttemptResult {
    Success,
    Failed,
    SkippedDuplicate,
    Timeout,
    WorkerError,
}

/// A single repair attempt record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepairAttemptRecord {
    pub theorem_name: String,
    pub goal_hash: String,
    pub stderr_hash: String,
    pub tactic_hash: String,
    pub tactic: String,
    pub strategy: RepairStrategy,
    pub source: RepairSource,
    pub confidence: f32,
    pub result: AttemptResult,
    pub output_hash: Option<String>,
}

impl RepairAttemptRecord {
    pub fn new(
        theorem: &str,
        goal: &str,
        stderr: &str,
        tactic: &str,
        strategy: RepairStrategy,
        source: RepairSource,
        result: AttemptResult,
    ) -> Self {
        use std::hash::{Hash, Hasher};

        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        goal.hash(&mut h1);
        let ghash = format!("{:x}", h1.finish());

        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        stderr.hash(&mut h2);
        let shash = format!("{:x}", h2.finish());

        let mut h3 = std::collections::hash_map::DefaultHasher::new();
        tactic.hash(&mut h3);
        let thash = format!("{:x}", h3.finish());

        Self {
            theorem_name: theorem.to_string(),
            goal_hash: ghash,
            stderr_hash: shash,
            tactic_hash: thash,
            tactic: tactic.to_string(),
            strategy,
            source,
            confidence: strategy.confidence(),
            result,
            output_hash: None,
        }
    }

    pub fn with_output_hash(mut self, hash: String) -> Self {
        self.output_hash = Some(hash);
        self
    }
}

/// Final status of CTRL repair attempt
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CtrlFinalStatus {
    Repaired,
    Unrepaired,
    Timeout,
    WorkerError,
}

/// Full audit trail for a CTRL repair session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CtrlAuditTrail {
    pub theorem_name: String,
    pub initial_failure_hash: String,
    pub attempts: Vec<RepairAttemptRecord>,
    pub final_status: CtrlFinalStatus,
    pub selected_repair: Option<String>, // tactic that succeeded
}

impl CtrlAuditTrail {
    pub fn new(theorem: &str, failure_hash: &str) -> Self {
        Self {
            theorem_name: theorem.to_string(),
            initial_failure_hash: failure_hash.to_string(),
            attempts: Vec::new(),
            final_status: CtrlFinalStatus::Unrepaired,
            selected_repair: None,
        }
    }

    pub fn add_attempt(&mut self, attempt: RepairAttemptRecord) {
        // Check if this was a successful or duplicate attempt
        let is_dup = matches!(attempt.result, AttemptResult::SkippedDuplicate);

        if !is_dup {
            self.attempts.push(attempt.clone());
        }

        // Update final status based on result
        if matches!(attempt.result, AttemptResult::Success) {
            self.final_status = CtrlFinalStatus::Repaired;
            self.selected_repair = Some(attempt.tactic.clone());
        } else if matches!(attempt.result, AttemptResult::Timeout) {
            self.final_status = CtrlFinalStatus::Timeout;
        } else if matches!(attempt.result, AttemptResult::WorkerError) {
            self.final_status = CtrlFinalStatus::WorkerError;
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get summary of attempts
    pub fn summary(&self) -> String {
        let success_count = self
            .attempts
            .iter()
            .filter(|a| matches!(a.result, AttemptResult::Success))
            .count();
        let failed_count = self
            .attempts
            .iter()
            .filter(|a| matches!(a.result, AttemptResult::Failed))
            .count();

        format!(
            "{} attempts: {} succeeded, {} failed, status: {:?}",
            self.attempts.len(),
            success_count,
            failed_count,
            self.final_status
        )
    }
}
