use std::collections::HashMap;
use crate::lean_error::LeanErrorKind;

#[derive(Debug, Clone)]
pub struct FailureStats {
    pub failed_count: u32,
    pub last_error: String,
    pub error_kind: LeanErrorKind,
}

pub struct FailureMemory {
    // (GoalFingerprint, TacticFingerprint) -> Stats
    pub memory: HashMap<(String, String), FailureStats>,
}

impl FailureMemory {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }

    pub fn record_failure(&mut self, goal_fp: String, tactic_fp: String, kind: LeanErrorKind, error: String) {
        let stats = self.memory.entry((goal_fp, tactic_fp)).or_insert(FailureStats {
            failed_count: 0,
            last_error: String::new(),
            error_kind: kind,
        });
        stats.failed_count += 1;
        stats.last_error = error;
        stats.error_kind = kind;
    }

    pub fn has_failed(&self, goal_fp: &str, tactic_fp: &str) -> bool {
        self.memory.contains_key(&(goal_fp.to_string(), tactic_fp.to_string()))
    }
    
    pub fn get_failure_count(&self, goal_fp: &str, tactic_fp: &str) -> u32 {
        self.memory.get(&(goal_fp.to_string(), tactic_fp.to_string()))
            .map(|s| s.failed_count)
            .unwrap_or(0)
    }
}
