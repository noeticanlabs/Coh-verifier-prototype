use crate::trajectory::types::{AdmissibleTrajectory, CandidateEdge};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierStats {
    pub total_expanded: usize,
    pub admissible_found: usize,
    pub rejected_found: usize,
    pub max_depth_reached: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub admissible: Vec<AdmissibleTrajectory>,
    pub rejected: Vec<CandidateEdge>,
    pub frontier_stats: FrontierStats,
    pub max_violation_seen: u128, // delta(r) max
}

impl SearchResult {
    pub fn new() -> Self {
        Self {
            admissible: Vec::new(),
            rejected: Vec::new(),
            frontier_stats: FrontierStats {
                total_expanded: 0,
                admissible_found: 0,
                rejected_found: 0,
                max_depth_reached: 0,
            },
            max_violation_seen: 0,
        }
    }
}
