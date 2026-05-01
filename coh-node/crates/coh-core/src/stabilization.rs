// fixture_only: allow_mock
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use num_traits::One;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StabilityState {
    Unstable,     // Oscillating or drifting
    Marginal,     // Barely controlled
    Stable,       // Bounded, predictable
    Overdamped,   // Too rigid (low adaptability)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilizationMetrics {
    pub window_steps: u64,

    pub margin_variance: Rational64,
    pub spinor_variance: Rational64,
    pub alignment_variance: Rational64,
    pub retrieval_variance: Rational64,

    pub compression_churn: Rational64,
    pub rephase_rate: Rational64,

    pub stability_score: Rational64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilizationThresholds {
    pub max_margin_variance: Rational64,
    pub max_spinor_variance: Rational64,
    pub max_alignment_variance: Rational64,
    pub max_retrieval_variance: Rational64,
    pub max_compression_churn: Rational64,
    pub max_rephase_rate: Rational64,
    pub min_stability_score: Rational64,
    pub overdamped_variance_floor: Rational64,
}

impl Default for StabilizationThresholds {
    fn default() -> Self {
        Self {
            max_margin_variance: Rational64::from_integer(1000),
            max_spinor_variance: Rational64::new(1, 10),
            max_alignment_variance: Rational64::new(1, 100),
            max_retrieval_variance: Rational64::from_integer(50),
            max_compression_churn: Rational64::new(1, 10),
            max_rephase_rate: Rational64::new(1, 100),
            min_stability_score: Rational64::from_integer(10),
            overdamped_variance_floor: Rational64::new(1, 1000000),
        }
    }
}

pub fn calculate_stabilization_score(m: &StabilizationMetrics) -> Rational64 {
    let variance_sum =
        m.margin_variance
        + m.spinor_variance
        + m.alignment_variance
        + m.retrieval_variance
        + m.compression_churn
        + m.rephase_rate
        + Rational64::new(1, 1000); // ε

    (Rational64::one() / variance_sum).reduced()
}

pub fn classify_stability(
    m: &StabilizationMetrics,
    t: &StabilizationThresholds,
) -> StabilityState {
    let score = calculate_stabilization_score(m);
    
    if score < t.min_stability_score {
        return StabilityState::Unstable;
    }

    if m.margin_variance > t.max_margin_variance 
        || m.spinor_variance > t.max_spinor_variance
        || m.alignment_variance > t.max_alignment_variance
        || m.retrieval_variance > t.max_retrieval_variance
    {
        return StabilityState::Marginal;
    }

    // Overdamped check: if variances are extremely low, the system might be too rigid
    if m.margin_variance < t.overdamped_variance_floor
        && m.spinor_variance < t.overdamped_variance_floor
        && m.alignment_variance < t.overdamped_variance_floor
    {
        return StabilityState::Overdamped;
    }

    StabilityState::Stable
}

pub struct StabilizationTracker {
    pub window_size: usize,
    pub margins: Vec<Rational64>,
    pub spinors: Vec<Rational64>, // Phase/Norm proxy
    pub alignments: Vec<Rational64>,
    pub retrieval_costs: Vec<Rational64>,
    pub compression_events: u64,
    pub rephase_events: u64,
}

impl StabilizationTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            margins: Vec::with_capacity(window_size),
            spinors: Vec::with_capacity(window_size),
            alignments: Vec::with_capacity(window_size),
            retrieval_costs: Vec::with_capacity(window_size),
            compression_events: 0,
            rephase_events: 0,
        }
    }

    pub fn record_step(
        &mut self,
        margin: Rational64,
        spin_phase: Rational64,
        alignment: Rational64,
        cost: Rational64,
    ) {
        if self.margins.len() >= self.window_size {
            self.margins.remove(0);
            self.spinors.remove(0);
            self.alignments.remove(0);
            self.retrieval_costs.remove(0);
        }
        self.margins.push(margin);
        self.spinors.push(spin_phase);
        self.alignments.push(alignment);
        self.retrieval_costs.push(cost);
    }

    pub fn record_event(&mut self, compressed: bool, rephased: bool) {
        if compressed { self.compression_events += 1; }
        if rephased { self.rephase_events += 1; }
    }

    fn variance(data: &[Rational64]) -> Rational64 {
        let n = data.len() as i64;
        if n < 2 { return Rational64::from_integer(0); }
        
        let mean = data.iter().sum::<Rational64>() / Rational64::from_integer(n);
        let var_sum: Rational64 = data.iter().map(|&x| {
            let diff = x - mean;
            diff * diff
        }).sum();
        
        (var_sum / Rational64::from_integer(n)).reduced()
    }

    pub fn compute_metrics(&self) -> StabilizationMetrics {
        let count = self.margins.len() as u64;
        let mut m = StabilizationMetrics {
            window_steps: count,
            margin_variance: Self::variance(&self.margins),
            spinor_variance: Self::variance(&self.spinors),
            alignment_variance: Self::variance(&self.alignments),
            retrieval_variance: Self::variance(&self.retrieval_costs),
            compression_churn: Rational64::new(self.compression_events as i64, count.max(1) as i64),
            rephase_rate: Rational64::new(self.rephase_events as i64, count.max(1) as i64),
            stability_score: Rational64::from_integer(0),
        };
        m.stability_score = calculate_stabilization_score(&m);
        m
    }
}
