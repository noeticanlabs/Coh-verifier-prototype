//! Trajectory Layer - V3 Geometry
//!
//! Implements the distance metric d(x, y) = inf { delta(tau) | tau : x -> y }

use crate::types::Hash32;

/// A state in the trajectory graph
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateNode {
    pub hash: Hash32,
    pub potential: u128,
}

/// A transition between states with its associated defect (delta)
#[derive(Clone, Debug)]
pub struct Transition {
    pub from: StateNode,
    pub to: StateNode,
    pub delta: u128,
    pub step_type: Option<String>,
}

/// A path of transitions (Trace)
pub struct Trajectory {
    pub steps: Vec<Transition>,
}

impl Trajectory {
    /// Calculate the total defect of the trajectory (Subadditivity law)
    pub fn total_defect(&self) -> u128 {
        self.steps.iter().map(|t| t.delta).sum()
    }
}
