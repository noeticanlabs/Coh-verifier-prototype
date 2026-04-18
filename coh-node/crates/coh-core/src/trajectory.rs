//! Trajectory Engine — Constrained search over admissible execution futures

use crate::reject::RejectCode;
use crate::types::{Decision, MicroReceiptWire};
use crate::verify_micro;
use std::collections::HashMap;

/// Witness status for constraint verification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessStatus {
    Pass,
    Fail,
    Unknown,
}

/// Witness map: constraint C1-C6 -> status
pub type WitnessMap = HashMap<String, WitnessStatus>;

/// A single step in a trajectory
#[derive(Debug, Clone)]
pub struct TrajStep {
    pub state: String,
    pub next_state: String,
    pub action: String,
    pub receipt: Option<MicroReceiptWire>,
    pub witnesses: WitnessMap,
}

/// A complete trajectory
#[derive(Debug, Clone)]
pub struct Trajectory {
    pub id: String,
    pub steps: Vec<TrajStep>,
    pub depth: usize,
    pub score: f64,
    pub is_selectable: bool,
    pub first_failure_index: Option<usize>,
}

impl Trajectory {
    pub fn new(id: String, _initial_state: String) -> Self {
        Self {
            id,
            steps: Vec::new(),
            depth: 0,
            score: 0.0,
            is_selectable: true,
            first_failure_index: None,
        }
    }

    pub fn push(&mut self, step: TrajStep) {
        let has_failure = step.witnesses.values().any(|v| *v == WitnessStatus::Fail);
        if has_failure && self.first_failure_index.is_none() {
            self.first_failure_index = Some(self.steps.len());
        }
        self.is_selectable = self.first_failure_index.is_none();
        self.steps.push(step);
        self.depth = self.steps.len();
    }

    pub fn last_state(&self) -> &str {
        self.steps
            .last()
            .map(|s| s.next_state.as_str())
            .unwrap_or("")
    }
}

/// Search context
#[derive(Debug, Clone)]
pub struct SearchContext {
    pub initial_state: String,
    pub max_depth: usize,
    pub beam_width: usize,
    pub weight_goal: f64,
    pub weight_risk: f64,
    pub weight_cost: f64,
    pub weight_uncertainty: f64,
}

impl Default for SearchContext {
    fn default() -> Self {
        Self {
            initial_state: "START".to_string(),
            max_depth: 5,
            beam_width: 3,
            weight_goal: 1.0,
            weight_risk: 0.5,
            weight_cost: 0.2,
            weight_uncertainty: 0.1,
        }
    }
}

/// Candidate action
#[derive(Debug, Clone)]
pub struct Candidate {
    pub state: String,
    pub next_state: String,
    pub action: String,
    pub receipt: Option<MicroReceiptWire>,
    pub witnesses: WitnessMap,
}

/// Enumerate actions (domain-specific)
pub fn enumerate_actions(_state: &str) -> Vec<Candidate> {
    vec![]
}

/// Verify edge against constraints
pub fn verify_edge(candidate: &Candidate) -> Result<Decision, String> {
    if let Some(receipt) = &candidate.receipt {
        let result = verify_micro(receipt.clone());
        let mut witnesses: WitnessMap = HashMap::new();

        match &result.code {
            Some(code) => match code {
                RejectCode::RejectSchema => {
                    witnesses.insert("C1".to_string(), WitnessStatus::Fail);
                }
                RejectCode::RejectMissingSignature | RejectCode::RejectMissingObjectId => {
                    witnesses.insert("C2".to_string(), WitnessStatus::Fail);
                }
                RejectCode::RejectCanonProfile => {
                    witnesses.insert("C3".to_string(), WitnessStatus::Fail);
                }
                RejectCode::RejectStateHashLink => {
                    witnesses.insert("C4".to_string(), WitnessStatus::Fail);
                }
                RejectCode::RejectChainDigest => {
                    witnesses.insert("C5".to_string(), WitnessStatus::Fail);
                }
                RejectCode::RejectPolicyViolation | RejectCode::SpendExceedsBalance => {
                    witnesses.insert("C6".to_string(), WitnessStatus::Fail);
                }
                _ => {
                    for c in ["C1", "C2", "C3", "C4", "C5", "C6"] {
                        witnesses.insert(c.to_string(), WitnessStatus::Unknown);
                    }
                }
            },
            None => {
                for c in ["C1", "C2", "C3", "C4", "C5", "C6"] {
                    witnesses.insert(c.to_string(), WitnessStatus::Pass);
                }
            }
        }
        return Ok(result.decision);
    }
    Err("No receipt".to_string())
}

/// Extend trajectory
fn extend_trajectory(traj: &Trajectory, step: TrajStep) -> Trajectory {
    let mut steps = traj.steps.clone();
    steps.push(step.clone());
    let depth = steps.len();
    let first_failure = traj.first_failure_index.or_else(|| {
        if step.witnesses.values().any(|v| *v == WitnessStatus::Fail) {
            Some(depth - 1)
        } else {
            None
        }
    });
    Trajectory {
        id: format!("{}+{}", traj.id, depth),
        steps,
        depth,
        score: 0.0,
        is_selectable: first_failure.is_none(),
        first_failure_index: first_failure,
    }
}

/// Score trajectory
fn score_trajectory(traj: &Trajectory, ctx: &SearchContext) -> f64 {
    let goal = traj.depth as f64;
    let risk = traj.first_failure_index.map(|i| i as f64).unwrap_or(0.0) * ctx.weight_risk;
    let cost = traj.depth as f64 * ctx.weight_cost;
    let uncertainty = traj
        .steps
        .iter()
        .filter(|s| s.witnesses.values().any(|v| *v == WitnessStatus::Unknown))
        .count() as f64
        * ctx.weight_uncertainty;
    ctx.weight_goal * goal - risk - cost - uncertainty
}

/// Beam search
pub fn search(initial_state: &str, max_depth: usize, beam_width: usize) -> Vec<Trajectory> {
    let ctx = SearchContext {
        initial_state: initial_state.to_string(),
        max_depth,
        beam_width,
        ..Default::default()
    };
    search_with_context(&ctx)
}

pub fn search_with_context(ctx: &SearchContext) -> Vec<Trajectory> {
    let mut frontier = vec![Trajectory::new(
        "ROOT".to_string(),
        ctx.initial_state.clone(),
    )];

    for _ in 0..ctx.max_depth {
        let mut next = Vec::new();
        for traj in frontier {
            let current = traj.last_state();
            if current.is_empty() {
                continue;
            }
            let candidates: Vec<_> = enumerate_actions(current)
                .into_iter()
                .take(ctx.beam_width)
                .collect();
            for candidate in candidates {
                let mut witnesses = candidate.witnesses.clone();
                if let Ok(decision) = verify_edge(&candidate) {
                    match decision {
                        Decision::Accept => {
                            for c in ["C1", "C2", "C3", "C4", "C5", "C6"] {
                                witnesses.insert(c.to_string(), WitnessStatus::Pass);
                            }
                        }
                        Decision::Reject => {}
                        _ => {
                            for c in ["C1", "C2", "C3", "C4", "C5", "C6"] {
                                witnesses.insert(c.to_string(), WitnessStatus::Pass);
                            }
                        }
                    }
                }
                let step = TrajStep {
                    state: candidate.state,
                    next_state: candidate.next_state,
                    action: candidate.action,
                    receipt: candidate.receipt,
                    witnesses,
                };
                next.push(extend_trajectory(&traj, step));
            }
        }
        for t in &mut next {
            t.score = score_trajectory(t, ctx);
        }
        next.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        frontier = next.into_iter().take(ctx.beam_width).collect();
        if frontier.is_empty() {
            break;
        }
    }
    frontier
}

/// Get admissible trajectories
pub fn get_admissible(trajectories: &[Trajectory]) -> Vec<&Trajectory> {
    trajectories.iter().filter(|t| t.is_selectable).collect()
}

/// Get best trajectory
pub fn get_best(trajectories: &[Trajectory]) -> Option<&Trajectory> {
    get_admissible(trajectories).into_iter().max_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trajectory_new() {
        let t = Trajectory::new("test".to_string(), "START".to_string());
        assert_eq!(t.id, "test");
        assert!(t.is_selectable);
    }
}
