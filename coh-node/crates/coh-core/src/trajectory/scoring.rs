use crate::trajectory::types::{AdmissibleTrajectory, DomainState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PathEvaluation {
    pub safety_bottleneck: f64,
    pub progress: f64,
    pub normalized_cost: f64,
}

impl Eq for PathEvaluation {}

impl PartialOrd for PathEvaluation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathEvaluation {
    /// Lexicographic comparison: Safety > Progress > -Cost
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 1. Safety Bottleneck (Min-Margin)
        let s_cmp = self.safety_bottleneck.partial_cmp(&other.safety_bottleneck).unwrap();
        if s_cmp != std::cmp::Ordering::Equal {
            return s_cmp;
        }

        // 2. Progress Index
        let p_cmp = self.progress.partial_cmp(&other.progress).unwrap();
        if p_cmp != std::cmp::Ordering::Equal {
            return p_cmp;
        }

        // 3. Inverse Cost (Minimize steps)
        other.normalized_cost.partial_cmp(&self.normalized_cost).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub goal: f64,
    pub risk: f64,
    pub cost: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            goal: 1.0,
            risk: 1.0, // Weight is less critical due to lexicographic override, but kept for scalar score
            cost: 0.1,
        }
    }
}

/// Compute evaluation metrics for a trajectory
pub fn evaluate_path(
    traj: &AdmissibleTrajectory,
    max_depth: usize,
) -> PathEvaluation {
    // 1. Minimum Safety Margin (Bottleneck)
    let safety_bottleneck = traj.steps.iter()
        .map(|s| s.state_next.safety_margin())
        .fold(1.0f64, |acc: f64, m: f64| acc.min(m));

    // 2. Local Progress Index of last state
    let progress = traj.steps.last()
        .map(|s| s.state_next.progress_index())
        .unwrap_or(0.0);

    // 3. Normalized Cost (|\tau| / K_max)
    let normalized_cost = if max_depth == 0 { 0.0 } else {
        traj.steps.len() as f64 / max_depth as f64
    };

    PathEvaluation {
        safety_bottleneck,
        progress,
        normalized_cost: normalized_cost.clamp(0.0, 1.0),
    }
}

/// Scalar weighted sum for UI display (Selection uses evaluate_path().cmp())
pub fn calculate_weighted_score(
    eval: &PathEvaluation,
    weights: &ScoringWeights,
) -> f64 {
    (eval.progress * weights.goal) + 
    (eval.safety_bottleneck * weights.risk) - 
    (eval.normalized_cost * weights.cost)
}
