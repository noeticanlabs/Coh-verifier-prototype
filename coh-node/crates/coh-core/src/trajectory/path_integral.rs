use crate::cohbit::CohBit;
use crate::atom::CohGovernor;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

/// Coh History: A sequence of bits forming a trajectory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohHistory {
    pub steps: Vec<CohBit>,
}

impl CohHistory {
    /// Total Action of the history: J(H) = sum J(e_i)
    pub fn total_action(&self, gov: &CohGovernor, lambda: f64, gauge_curvature: f64) -> f64 {
        self.steps.iter().map(|b| gov.compute_action(b, lambda, gauge_curvature)).sum()
    }

    /// Path Probability: P(H) = e^{-J(H)/tau} * Product(sigmoid(beta * m_i))
    pub fn path_probability(&self, gov: &CohGovernor, lambda: f64, gauge_curvature: f64, tau: f64, beta: f64) -> f64 {
        let action = self.total_action(gov, lambda, gauge_curvature);
        let weight = (-action / tau).exp();
        
        let mut gate_product = 1.0;
        for b in &self.steps {
            let m = b.margin().to_f64().unwrap_or(0.0);
            let gate = 1.0 / (1.0 + (-beta * m).exp());
            gate_product *= gate;
        }
        
        weight * gate_product
    }
}

/// Propagator: Manages the evaluation of multiple histories.
pub struct Propagator;

impl Propagator {
    /// Compute Partition Function Z = sum P(H)
    pub fn partition_function(histories: &[CohHistory], gov: &CohGovernor, lambda: f64, gauge_curvature: f64, tau: f64, beta: f64) -> f64 {
        histories.iter().map(|h| h.path_probability(gov, lambda, gauge_curvature, tau, beta)).sum()
    }
}
