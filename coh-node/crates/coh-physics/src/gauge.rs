//! Coh Yang-Mills - Non-Abelian Gauge & Curvature
//!
//! "Curvature is the failure of constraints to commute. 
//! High curvature marks regions of intense constraint conflict."

use serde::{Deserialize, Serialize};

/// Coh Gauge Field: A_\mu^a
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohGaugeField {
    pub dim: usize,
    /// Connection components: [mu][a]
    pub connection: [[f64; 8]; 4], // Support up to 8 generators (e.g. SU(3))
}

impl CohGaugeField {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            connection: [[0.0; 8]; 4],
        }
    }

    /// Compute Yang-Mills Curvature: F_munu^a = d_mu A_nu - d_nu A_mu + [A_mu, A_nu]
    /// [SCAFFOLD] Currently returns 0.0 as dynamics are still pending.
    pub fn curvature_at(&self, _mu: usize, _nu: usize, _a: usize) -> f64 {
        // In a real simulation, this would be computed from the field gradients
        // For the Coh kernel, we treat curvature as a given constraint conflict metric
        0.0
    }
}

/// Yang-Mills Curvature Tensor: F_munu
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YangMillsCurvature {
    pub dim: usize,
    pub f: [[[f64; 8]; 4]; 4], // [mu][nu][a]
}

impl YangMillsCurvature {
    /// Compute Tr(F^2) = Tr(F_munu F^munu)
    pub fn action_density(&self) -> f64 {
        let mut sum = 0.0;
        for mu in 0..4 {
            for nu in 0..4 {
                for a in 0..self.dim {
                    sum += self.f[mu][nu][a] * self.f[mu][nu][a];
                }
            }
        }
        sum
    }
}

/// Wilson Loop Receipt: Order-sensitive path-ordered receipt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WilsonLoopReceipt {
    pub path_hash: String,
    pub holonomy_trace: f64,
    pub curvature_sum: f64,
}

impl WilsonLoopReceipt {
    /// Compute Holonomy: W = Tr(P exp(i int A))
    /// In this simplified runtime model, we simulate the path-ordered product
    /// of local verifier rotations (gauge transformations).
    pub fn compute_holonomy(history: &coh_core::trajectory::path_integral::CohHistory, gauge: &CohGaugeField) -> f64 {
        let mut total_phase = 0.0;
        
        // Sum the gauge connection along the path-ordered steps
        for _bit in &history.steps {
            // Pick a representative component (e.g. mu=0, a=0)
            total_phase += gauge.connection[0][0];
        }
        
        // Trace of rotation matrix in U(1) scaffold
        (total_phase.cos() + total_phase.sin()) / 2.0
    }

    /// Is Holonomy Admissible: |1 - W| < epsilon
    pub fn is_admissible(&self, tolerance: f64) -> bool {
        (1.0 - self.holonomy_trace).abs() < tolerance
    }
}
