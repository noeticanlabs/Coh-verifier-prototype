use crate::gauge::CohGaugeField;
use coh_core::atom::CohGovernor;
use coh_core::cohbit::CohBit;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

/// Coh Field: A collection of interacting Coh Governors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohField {
    pub governors: Vec<CohGovernor>,
    pub global_gauge: CohGaugeField,
    pub coupling_constant: f64,
}

impl CohField {
    pub fn new(coupling: f64) -> Self {
        Self {
            governors: vec![],
            global_gauge: CohGaugeField::new(3),
            coupling_constant: coupling,
        }
    }

    pub fn interaction_cost(&self, gov_i: &CohGovernor, bit_i: &CohBit) -> f64 {
        let _current_i = bit_i.valuation_post.to_f64().unwrap_or(0.0);

        let neighbor_density: f64 = self
            .governors
            .iter()
            .filter(|g| g.state_hash != gov_i.state_hash)
            .map(|g| g.valuation.to_f64().unwrap_or(0.0))
            .sum();

        self.coupling_constant * neighbor_density
    }

    pub fn step(&mut self, lambda: f64, candidates: &[Vec<CohBit>]) -> Vec<(usize, String)> {
        let mut transitions = vec![];

        for i in 0..self.governors.len() {
            let int_cost = 0.1;
            let bits = &candidates[i];

            if let Some(optimal_bit) = self.governors[i].select_optimal_bit(bits, lambda, int_cost).cloned() {
                transitions.push((i, optimal_bit.action_hash.to_hex()));
                self.governors[i].evolve(&optimal_bit);
            }
        }

        transitions
    }
}
