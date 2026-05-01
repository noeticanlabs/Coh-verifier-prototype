// fixture_only: allow_mock
use serde::{Deserialize, Serialize};
use num_rational::Rational64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IgnitionState {
    Cold,
    Warming,
    Ignited,
    Quenched,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnitionMetrics {
    pub window_steps: u64,

    pub avg_margin: Rational64,
    pub avg_anchor_alignment: Rational64,
    pub avg_memory_usefulness: Rational64,
    pub avg_spinor_stability: Rational64,

    pub avg_defect_pressure: Rational64,
    pub avg_operating_cost: Rational64,

    pub cone_exits: u64,
    pub rephases: u64,
    pub invalid_rejections: u64,

    pub ignition_score: Rational64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnitionThresholds {
    pub min_score: Rational64,
    pub min_avg_margin: Rational64,
    pub min_anchor_alignment: Rational64,
    pub min_memory_usefulness: Rational64,
    pub min_spinor_stability: Rational64,
    pub max_defect_pressure: Rational64,
    pub max_operating_cost: Rational64,
    pub max_cone_exits: u64,
    pub max_rephases: u64,
}

impl Default for IgnitionThresholds {
    fn default() -> Self {
        Self {
            min_score: Rational64::from_integer(100),
            min_avg_margin: Rational64::from_integer(10),
            min_anchor_alignment: Rational64::new(80, 100),
            min_memory_usefulness: Rational64::new(10, 100),
            min_spinor_stability: Rational64::new(90, 100),
            max_defect_pressure: Rational64::from_integer(50),
            max_operating_cost: Rational64::from_integer(20),
            max_cone_exits: 5,
            max_rephases: 2,
        }
    }
}

pub fn calculate_ignition_score(m: &IgnitionMetrics) -> Rational64 {
    let numerator =
        m.avg_margin
        * m.avg_anchor_alignment
        * m.avg_memory_usefulness
        * m.avg_spinor_stability;

    let denominator =
        m.avg_defect_pressure
        + m.avg_operating_cost
        + Rational64::new(1, 1000); // ε to prevent div by zero

    (numerator / denominator).reduced()
}

pub fn classify_ignition(
    m: &IgnitionMetrics,
    t: &IgnitionThresholds,
) -> IgnitionState {
    if m.cone_exits > t.max_cone_exits || m.rephases > t.max_rephases {
        return IgnitionState::Quenched;
    }

    if m.avg_margin < t.min_avg_margin {
        return IgnitionState::Cold;
    }

    if m.ignition_score >= t.min_score
        && m.avg_anchor_alignment >= t.min_anchor_alignment
        && m.avg_memory_usefulness >= t.min_memory_usefulness
        && m.avg_spinor_stability >= t.min_spinor_stability
        && m.avg_defect_pressure <= t.max_defect_pressure
        && m.avg_operating_cost <= t.max_operating_cost
    {
        return IgnitionState::Ignited;
    }

    IgnitionState::Warming
}

/// Tracking buffer for calculating windowed metrics
pub struct IgnitionTracker {
    pub window_size: usize,
    pub margins: Vec<Rational64>,
    pub alignments: Vec<Rational64>,
    pub memory_hits: Vec<Rational64>,
    pub stability: Vec<Rational64>,
    pub defects: Vec<Rational64>,
    pub costs: Vec<Rational64>,
    
    pub cone_exits: u64,
    pub rephases: u64,
    pub invalid_rejections: u64,
}

impl IgnitionTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            margins: Vec::with_capacity(window_size),
            alignments: Vec::with_capacity(window_size),
            memory_hits: Vec::with_capacity(window_size),
            stability: Vec::with_capacity(window_size),
            defects: Vec::with_capacity(window_size),
            costs: Vec::with_capacity(window_size),
            cone_exits: 0,
            rephases: 0,
            invalid_rejections: 0,
        }
    }

    pub fn record_step(
        &mut self,
        margin: Rational64,
        alignment: Rational64,
        memory_useful: Rational64,
        spin_stability: Rational64,
        defect: Rational64,
        cost: Rational64,
    ) {
        if self.margins.len() >= self.window_size {
            self.margins.remove(0);
            self.alignments.remove(0);
            self.memory_hits.remove(0);
            self.stability.remove(0);
            self.defects.remove(0);
            self.costs.remove(0);
        }
        self.margins.push(margin);
        self.alignments.push(alignment);
        self.memory_hits.push(memory_useful);
        self.stability.push(spin_stability);
        self.defects.push(defect);
        self.costs.push(cost);
    }

    pub fn record_event(&mut self, cone_exit: bool, rephase: bool, rejected: bool) {
        if cone_exit { self.cone_exits += 1; }
        if rephase { self.rephases += 1; }
        if rejected { self.invalid_rejections += 1; }
    }

    pub fn compute_metrics(&self) -> IgnitionMetrics {
        let count = self.margins.len() as i64;
        if count == 0 {
            return IgnitionMetrics {
                window_steps: 0,
                avg_margin: Rational64::from_integer(0),
                avg_anchor_alignment: Rational64::from_integer(0),
                avg_memory_usefulness: Rational64::from_integer(0),
                avg_spinor_stability: Rational64::from_integer(0),
                avg_defect_pressure: Rational64::from_integer(0),
                avg_operating_cost: Rational64::from_integer(0),
                cone_exits: self.cone_exits,
                rephases: self.rephases,
                invalid_rejections: self.invalid_rejections,
                ignition_score: Rational64::from_integer(0),
            };
        }

        let avg = |v: &Vec<Rational64>| v.iter().sum::<Rational64>() / Rational64::from_integer(count);

        let mut m = IgnitionMetrics {
            window_steps: count as u64,
            avg_margin: avg(&self.margins),
            avg_anchor_alignment: avg(&self.alignments),
            avg_memory_usefulness: avg(&self.memory_hits),
            avg_spinor_stability: avg(&self.stability),
            avg_defect_pressure: avg(&self.defects),
            avg_operating_cost: avg(&self.costs),
            cone_exits: self.cone_exits,
            rephases: self.rephases,
            invalid_rejections: self.invalid_rejections,
            ignition_score: Rational64::from_integer(0),
        };
        m.ignition_score = calculate_ignition_score(&m);
        m
    }
}
