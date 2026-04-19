use crate::trajectory::types::{AdmissibleTrajectory, DomainState};
use crate::trajectory::domain::goal_distance;

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
            risk: 0.5,
            cost: 0.1,
        }
    }
}

pub fn calculate_score(
    traj: &AdmissibleTrajectory,
    target: &DomainState,
    weights: &ScoringWeights,
) -> f64 {
    let last_state = traj.steps.last().map(|s| &s.state_next);
    
    // 1. Goal distance
    let dist = last_state.map(|s| goal_distance(s, target)).unwrap_or(1.0);
    let goal_score = (1.0 - dist) * weights.goal;
    
    // 2. Risk margin (how close are we to balance limits etc)
    let risk_score = calculate_risk_margin(traj) * weights.risk;
    
    // 3. Step cost
    let step_cost = traj.steps.len() as f64 * weights.cost;
    
    goal_score + risk_score - step_cost
}

fn calculate_risk_margin(traj: &AdmissibleTrajectory) -> f64 {
    // Heuristic: lower balance = higher risk (lower margin)
    let mut margin = 1.0;
    for step in &traj.steps {
        if let DomainState::Financial(fs) = &step.state_next {
            if fs.balance < 1000 {
                margin *= 0.8;
            }
        }
    }
    margin
}
