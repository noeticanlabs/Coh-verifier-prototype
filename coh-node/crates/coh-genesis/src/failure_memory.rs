use crate::lean_error::LeanErrorKind;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FailureStats {
    pub failed_count: u32,
    pub last_error: String,
    pub error_kind: LeanErrorKind,
}

/// Key for deduplicating attempt tracking
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AttemptKey {
    pub theorem_name: String,
    pub goal_hash: String,
    pub stderr_hash: String,
    pub tactic_hash: String,
}

impl AttemptKey {
    pub fn new(theorem: &str, goal: &str, stderr: &str, tactic: &str) -> Self {
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
        }
    }
}

pub struct FailureMemory {
    // (GoalFingerprint, TacticFingerprint) -> Stats
    pub memory: HashMap<(String, String), FailureStats>,
    // Attempt tracking for deduplication
    pub attempts: HashMap<AttemptKey, bool>,
}

impl FailureMemory {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
            attempts: HashMap::new(),
        }
    }

    pub fn record_failure(
        &mut self,
        goal_fp: String,
        tactic_fp: String,
        kind: LeanErrorKind,
        error: String,
    ) {
        let stats = self
            .memory
            .entry((goal_fp, tactic_fp))
            .or_insert(FailureStats {
                failed_count: 0,
                last_error: String::new(),
                error_kind: kind,
            });
        stats.failed_count += 1;
        stats.last_error = error;
        stats.error_kind = kind;
    }

    pub fn has_failed(&self, goal_fp: &str, tactic_fp: &str) -> bool {
        self.memory
            .contains_key(&(goal_fp.to_string(), tactic_fp.to_string()))
    }

    pub fn get_failure_count(&self, goal_fp: &str, tactic_fp: &str) -> u32 {
        self.memory
            .get(&(goal_fp.to_string(), tactic_fp.to_string()))
            .map(|s| s.failed_count)
            .unwrap_or(0)
    }
}
