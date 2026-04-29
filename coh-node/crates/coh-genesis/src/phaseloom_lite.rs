//! PhaseLoomLite: Receipt-grounded adaptive memory for NPE strategy selection.
//!
//! PhaseLoom consumes boundary receipts from Coh verification and biases future NPE proposals.
//! **Safety Invariant**: PhaseLoom may bias proposals but never bypasses verification.
//!
//! ## State
//!
//! - `strategy_weights`: Learned weights for strategy classes (normalized to sum = 1.0)
//! - `curvature`: Accumulated rejection stress (increases on rejection)
//! - `budget`: Remaining thermodynamic work capacity
//! - `tau`: Intrinsic step counter
//!
//! ## Update Law
//!
//! ```
//! w_{c,n+1} = Normalize(w_{c,n} + η × R_c - ρ × F_c)
//! C_{n+1} = C_n + (rejected outcomes)
//! B_{n+1} = B_n - spend
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for PhaseLoom initialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseLoomConfig {
    /// Initial budget (thermodynamic work capacity)
    pub initial_budget: u128,
    /// Learning rate (positive reward magnitude)
    pub learning_rate: f64,
    /// Curvature penalty coefficient
    pub curvature_penalty: f64,
    /// Circuit breaker threshold (max curvature before pause)
    pub circuit_break_threshold: u128,
    /// Minimum weight value (prevent weight collapse)
    pub min_weight: f64,
    /// Initial temperature for Boltzmann exploration
    pub initial_temperature: f64,
    /// Decay rate for temperature per step
    pub temperature_decay: f64,
    /// Minimum temperature floor
    pub min_temperature: f64,
    /// Target entropy floor for exploration
    pub entropy_floor: f64,
    /// Max exploration failures before forcing exploitation
    pub exploration_failure_threshold: u64,
    /// Time dilation coefficient for Task Error
    pub alpha_v: f64,
    /// Time dilation coefficient for Tension
    pub alpha_t: f64,
    /// Time dilation coefficient for Curvature
    pub alpha_c: f64,
    /// [ECOLOGY] Temporal Depth coefficient (alpha_tau)
    pub alpha_tau: f64,
    /// [ECOLOGY] Semantic Distance coefficient (alpha_d)
    pub alpha_d: f64,
    /// [ECOLOGY] Provenance Distance coefficient (alpha_p)
    pub alpha_p: f64,
    /// [ECOLOGY] Maintenance cost coefficient (M)
    pub maintenance_coeff: f64,
}

impl Default for PhaseLoomConfig {
    fn default() -> Self {
        Self {
            initial_budget: 100_000,
            learning_rate: 0.1,
            curvature_penalty: 0.05,
            circuit_break_threshold: 10_000,
            min_weight: 0.01,
            initial_temperature: 1.0,
            temperature_decay: 0.99,
            min_temperature: 0.1,
            entropy_floor: 0.5,
            exploration_failure_threshold: 5,
            alpha_v: 0.1,
            alpha_t: 0.2,
            alpha_c: 0.05,
            alpha_tau: 0.01,
            alpha_d: 1.0,
            alpha_p: 5.0,
            maintenance_coeff: 0.001,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StrategyWeights(pub HashMap<String, f64>);

impl StrategyWeights {
    /// Normalize weights to sum = 1.0 (probability distribution)
    pub fn normalize(&mut self) {
        let sum: f64 = self.0.values().sum();
        if sum > 0.0 {
            for value in self.0.values_mut() {
                *value /= sum;
            }
        }
    }

    /// Get normalized weight for a class
    pub fn get(&self, class: &str) -> f64 {
        self.0.get(class).copied().unwrap_or(0.0)
    }

    /// Increment weight for a class by delta
    pub fn increment(&mut self, class: &str, delta: f64) {
        self.0
            .entry(class.to_string())
            .and_modify(|w| *w = (*w + delta).max(0.0))
            .or_insert(delta.max(0.0));
    }

    /// Get all strategy classes
    pub fn classes(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }
}

/// PhaseLoom state: adaptive memory for NPE strategy selection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseLoomState {
    /// Strategy weight vector (bias toward proven useful outcomes)
    pub strategy_weights: StrategyWeights,
    /// Template weight vector (bias toward proven Coh patterns)
    pub template_weights: StrategyWeights,
    /// Structural curvature (accumulated rejection stress)
    pub curvature: u128,
    /// Remaining budget (thermodynamic work capacity)
    pub budget: u128,
    /// Intrinsic machine time (step counter)
    pub tau: u64,
    /// Count of accepted outcomes
    pub accepted_count: u64,
    /// Count of rejected outcomes
    pub rejected_count: u64,
    /// Failure counts per strategy class
    pub failure_counts: HashMap<String, u64>,
    /// Circuit breaker triggered
    pub circuit_broken: bool,
    /// Current temperature for Boltzmann exploration (decays over tau)
    pub temperature: f64,
    /// Consecutive exploration failures (triggers exploitation mode)
    pub exploration_failure_count: u64,
    /// Current entropy of strategy distribution
    pub current_entropy: f64,
    /// Whether to force exploitation (overrides entropy floor)
    pub force_exploitation: bool,
    /// Algebraic Tension T(x): Accumulated consistency violations
    pub tension: u128,
    /// Intrinsic PhaseLoom Time (dilated by tension)
    pub tau_f: f64,
    /// Epistemic Provenance Index (counts of sources)
    pub provenance_index: HashMap<String, u64>,
    /// Counters for summary
    pub closed_proofs: u64,
    pub build_passed_with_sorry: u64,
    pub near_misses: u64,
    pub max_tension: u128,
    pub dilation_events: u64,
    /// Detailed stats per template: (successes, failures)
    pub template_stats: HashMap<String, (u64, u64)>,
}

impl Default for PhaseLoomState {
    fn default() -> Self {
        Self::new(&PhaseLoomConfig::default())
    }
}

impl PhaseLoomState {
    /// Create new PhaseLoomState with configuration
    pub fn new(config: &PhaseLoomConfig) -> Self {
        Self {
            strategy_weights: StrategyWeights::default(),
            template_weights: StrategyWeights::default(),
            curvature: 0,
            budget: config.initial_budget,
            tau: 0,
            accepted_count: 0,
            rejected_count: 0,
            failure_counts: HashMap::new(),
            circuit_broken: false,
            temperature: config.initial_temperature,
            exploration_failure_count: 0,
            current_entropy: 0.0,
            force_exploitation: false,
            tension: 0,
            tau_f: 0.0,
            provenance_index: HashMap::new(),
            closed_proofs: 0,
            build_passed_with_sorry: 0,
            near_misses: 0,
            max_tension: 0,
            dilation_events: 0,
            template_stats: HashMap::new(),
        }
    }

    /// [ECOLOGY] Get Epistemic Authority rank: EXT > DER > REP > SIM
    pub fn provenance_authority(prov: &str) -> u8 {
        match prov {
            "EXT" => 4,
            "DER" => 3,
            "REP" => 2,
            "SIM" => 1,
            _ => 0,
        }
    }

    /// [ECOLOGY] Calculate monotonic read cost for a record
    pub fn calculate_read_cost(
        &self,
        config: &PhaseLoomConfig,
        record_tau: u64,
        semantic_dist: f64,
        record_prov: &str,
    ) -> u128 {
        let dt = self.tau.saturating_sub(record_tau) as f64;
        let dp = (Self::provenance_authority("EXT") as i8 - Self::provenance_authority(record_prov) as i8).abs() as f64;
        
        let cost = (config.alpha_tau * dt) + (config.alpha_d * semantic_dist) + (config.alpha_p * dp);
        cost.max(0.0) as u128
    }

    /// Compute Shannon entropy of strategy distribution: H = -Σ p * log(p)
    pub fn compute_entropy(&self) -> f64 {
        let weights = &self.strategy_weights.0;
        if weights.is_empty() {
            return 0.0;
        }

        let sum: f64 = weights.values().sum();
        if sum == 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for w in weights.values() {
            let p = w / sum;
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }
        entropy
    }

    /// [ECOLOGY] Verify memory transition against Provenance Lattice
    pub fn validate_memory_transition(&self, new_prov: &str, old_prov: &str) -> bool {
        let new_rank = Self::provenance_authority(new_prov);
        let old_rank = Self::provenance_authority(old_prov);
        
        // Theorem E3: The Anchor Firewall
        // Higher authority cannot be overwritten by lower authority without receipt.
        new_rank >= old_rank
    }

    /// Update temperature with decay schedule
    pub fn update_temperature(&mut self, config: &PhaseLoomConfig) {
        // Decay temperature, floor at min_temperature
        self.temperature =
            (self.temperature * config.temperature_decay).max(config.min_temperature);
    }

    /// Check if entropy floor is satisfied
    pub fn satisfies_entropy_floor(&self, config: &PhaseLoomConfig) -> bool {
        self.current_entropy >= config.entropy_floor || self.strategy_weights.0.len() < 2
    }

    /// Sample a strategy using Boltzmann exploration with entropy floor
    /// Returns (strategy_name, was_exploration)
    pub fn sample_strategy<R: rand::Rng>(
        &self,
        config: &PhaseLoomConfig,
        rng: &mut R,
    ) -> (Option<String>, bool) {
        // Force exploitation if circuit broken or too many exploration failures
        if self.force_exploitation || self.circuit_broken {
            // Exploitation: pick highest weight
            let best = self
                .strategy_weights
                .0
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(k, _)| k.clone());
            return (best, false);
        }

        let weights = &self.strategy_weights.0;
        if weights.is_empty() {
            return (None, false);
        }

        // Check entropy floor - if violated, force exploration
        let should_explore = !self.satisfies_entropy_floor(config)
            || self.exploration_failure_count >= config.exploration_failure_threshold;

        if should_explore || self.temperature > config.min_temperature {
            return (self.sample_boltzmann(rng), true);
        }

        (self.sample_best(), false)
    }

    /// Sample strategy using Boltzmann exploration
    pub fn sample_boltzmann<R: rand::Rng>(&self, rng: &mut R) -> Option<String> {
        let weights = &self.strategy_weights.0;
        let mut sum = 0.0;
        let mut exp_weights = Vec::new();
        for (class, weight) in weights {
            let exp_w = (weight / self.temperature.max(0.001)).exp();
            sum += exp_w;
            exp_weights.push((class, exp_w));
        }

        if sum > 0.0 {
            let r = rng.gen_range(0.0..sum);
            let mut acc = 0.0;
            for (class, exp_w) in exp_weights {
                acc += exp_w;
                if r <= acc {
                    return Some(class.clone());
                }
            }
        }

        self.sample_best()
    }

    /// Sample strategy with highest weight (exploitation only)
    pub fn sample_best(&self) -> Option<String> {
        if self.strategy_weights.0.is_empty() {
            return None;
        }

        self.strategy_weights
            .0
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k.clone())
    }

    /// Check circuit breaker - pause learning if curvature too high or budget exhausted
    pub fn is_circuit_broken(&self, config: &PhaseLoomConfig) -> bool {
        self.curvature > config.circuit_break_threshold || self.budget == 0
    }

    /// Process a boundary receipt and update internal state
    /// This implements the PhaseLoom Framework update laws
    pub fn ingest(&mut self, receipt: &BoundaryReceiptSummary, config: &PhaseLoomConfig) {
        // [PHASELOOM: PART III] Intrinsic Time Dilation
        // d tau / dt = 1 + alpha_V V + alpha_T T + alpha_C C
        let v_norm = receipt.genesis_margin.abs() as f64 / 1000.0;
        let t_norm = receipt.tension_score as f64 / 100.0;
        let c_norm = self.curvature as f64 / 1000.0;
        
        let d_tau = 1.0 + (config.alpha_v * v_norm) + (config.alpha_t * t_norm) + (config.alpha_c * c_norm);
        self.tau_f += d_tau;
        self.tau = self.tau_f as u64;

        if t_norm > 0.0 {
            self.dilation_events += 1;
            self.max_tension = self.max_tension.max(receipt.tension_score);
        }

        // [PHASELOOM: PART IV] Epistemic Firewall
        self.provenance_index
            .entry(receipt.provenance.clone())
            .and_modify(|c| *c = c.saturating_add(1))
            .or_insert(1);

        // [PHASELOOM: PART I] Tension Injection
        self.tension = self.tension.saturating_add(receipt.tension_score);

        // [PHASELOOM: PART VI] Weight Reinforcement
        // Implement the "ClosedNoSorry" policy
        let weight_delta = if receipt.accepted {
            if !receipt.sorry_detected && receipt.outcome == "accepted" {
                self.closed_proofs += 1;
                5.0 // ClosedNoSorry
            } else if receipt.sorry_detected {
                self.build_passed_with_sorry += 1;
                0.5 // BuildPassedWithSorry
            } else {
                self.near_misses += 1;
                1.0 // NearMiss
            }
        } else {
            if receipt.outcome == "rejected" {
                -1.0
            } else if receipt.outcome == "erroneous" {
                -2.0 // Malformed
            } else {
                -0.5 // Timeout or other
            }
        };

        if let Some(template) = receipt.coh_template {
            let t_name = template.as_str().to_string();
            self.template_weights.increment(&t_name, weight_delta);
            
            let stats = self.template_stats.entry(t_name).or_insert((0, 0));
            if receipt.accepted && !receipt.sorry_detected && receipt.outcome == "accepted" {
                stats.0 += 1; // Success
            } else if !receipt.accepted {
                stats.1 += 1; // Failure
            }
        }

        // Curvature accumulation (rejection penalty)
        if !receipt.accepted {
            self.curvature = self.curvature.saturating_add(1);
            self.rejected_count = self.rejected_count.saturating_add(1);

            // Track failure count per strategy class
            self.failure_counts
                .entry(receipt.strategy_class.clone())
                .and_modify(|c| *c = c.saturating_add(1))
                .or_insert(1);
        } else {
            self.accepted_count = self.accepted_count.saturating_add(1);
        }

        // Budget burn (work consumption)
        let mut spend: u128 = if receipt.accepted { 10 } else { 50 };
        
        // [PHASELOOM ECOLOGY: Lawful Recall]
        // Deduct read cost if this was a memory access
        if receipt.record_tau > 0 || receipt.semantic_distance > 0.0 {
            let read_cost = self.calculate_read_cost(config, receipt.record_tau, receipt.semantic_distance, &receipt.provenance);
            spend = spend.saturating_add(read_cost);
        }
        
        self.budget = self.budget.saturating_sub(spend);

        // Strategy weight update: reward acceptance, penalize failure
        let class = &receipt.strategy_class;
        let reward = if receipt.accepted {
            config.learning_rate * receipt.novelty.max(0.1)
        } else {
            // Use taxonomy-driven reward signal if available
            receipt
                .failure_report
                .as_ref()
                .map(|r| r.severity.reward_signal())
                .unwrap_or(0.0)
                * config.learning_rate
        };

        let penalty = if !receipt.accepted {
            let failure_count = self.failure_counts.get(class).copied().unwrap_or(1) as f64;
            config.curvature_penalty * failure_count.min(0.5)
        } else {
            0.0
        };

        self.strategy_weights.increment(class, reward - penalty);
        self.strategy_weights.normalize();

        // Template weight update: reward Coh pattern effectiveness
        if let Some(template) = &receipt.coh_template {
            let template_str = template.as_str();
            self.template_weights.increment(template_str, reward - penalty);
            self.template_weights.normalize();
        }

        // [PHASELOOM ECOLOGY: Metabolic Forgetting (Theorem E2)]
        // Prune memories (weights) whose utility falls below the maintenance cost threshold
        self.strategy_weights.0.retain(|_, &mut w| w >= config.maintenance_coeff);
        if self.strategy_weights.0.is_empty() {
            // Prevent complete amnesia of the active class
            self.strategy_weights.increment(class, config.min_weight);
            self.strategy_weights.normalize();
        }
        self.template_weights.0.retain(|_, &mut w| w >= config.maintenance_coeff);
        if !self.template_weights.0.is_empty() {
            self.template_weights.normalize();
        }

        // [NPE-Rust Advances NPE-Lean] Trigger synthesis on warm proof failure
        if let Some(report) = &receipt.failure_report {
            if let crate::failure_taxonomy::FailureKind::LeanProof(
                crate::failure_taxonomy::LeanProofFailure::UnsolvedGoals,
            ) = &report.kind
            {
                if reward > 0.0 {
                    println!("PHASELOOM: Warm proof failure detected. Suggesting SynthesisRepair for target '{}'.", receipt.target);
                }
            }
        }

        // Check circuit breaker
        self.circuit_broken = self.is_circuit_broken(config);
    }

    /// Get weight for a specific strategy class
    pub fn weight_for(&self, class: &str) -> f64 {
        self.strategy_weights.get(class)
    }

    /// Get all strategy classes with weights
    pub fn all_weights(&self) -> &HashMap<String, f64> {
        &self.strategy_weights.0
    }

    /// Serialize state to JSON bytes
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize state from JSON bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Boundary receipt summary consumed by PhaseLoom
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BoundaryReceiptSummary {
    /// Domain (e.g., "code", "test", "docs")
    pub domain: String,
    /// Target (e.g., "function foo", "module bar")
    pub target: String,
    /// Strategy class used (e.g., "synthesize", "refine", "debug")
    pub strategy_class: String,
    /// Coh Template used (e.g., "CertifiedComposition")
    pub coh_template: Option<crate::npe::templates::CohTemplateKind>,
    /// Wildness parameter (0.0 = conservative, 1.0 = aggressive)
    pub wildness: f64,
    /// Genesis margin: M(g') + C(p) - M(g) - D(p)
    pub genesis_margin: i128,
    /// Coherence margin: V_post + spend - V_pre - defect
    pub coherence_margin: i128,
    /// Projection Defect: Coarse-graining slack accounted for in memory
    pub projection_defect: u128,
    /// Algebraic Tension Score (0-100)
    pub tension_score: u128,
    /// Epistemic Provenance: "EXT", "DER", "REP", "SIM"
    pub provenance: String,
    /// [ECOLOGY] Tau of the original record being accessed
    pub record_tau: u64,
    /// [ECOLOGY] Semantic distance to the record
    pub semantic_distance: f64,
    /// Accuracy (Fiber Diameter): lower is more precise
    pub accuracy: f64,
    /// Utility Score for metabolic forgetting
    pub utility: f64,
    /// Whether sorry or admit was detected in the proof
    pub sorry_detected: bool,
    /// First failure reason if rejected
    pub first_failure: String,
    /// Outcome: "accepted", "rejected", "erroneous"
    pub outcome: String,
    /// Accepted: true/false
    pub accepted: bool,
    /// Novelty score (0.0 = repeat, 1.0 = novel)
    pub novelty: f64,
    /// Receipt hash for audit trail
    pub receipt_hash: String,
    /// Detailed failure report from the NPE pipeline
    pub failure_report: Option<crate::failure_taxonomy::FailureReport>,
    // ==== Mathlib fields for PhaseLoom learning ====
    /// Mathlib strategy used (e.g., "IsGLB", "SInf", "OrderTheory")
    pub mathlib_strategy: Option<String>,
    /// Mathlib confidence (0.0 - 1.0)
    pub mathlib_confidence: Option<f64>,
    /// Suggested lemmas from mathlib
    pub mathlib_suggested_lemmas: Option<Vec<String>>,
    /// Mathlib import risk tier
    pub mathlib_import_risk: Option<String>,
    /// Were imports actually used in proof?
    pub mathlib_imports_used: bool,
    /// What mathlib effect was observed
    pub mathlib_effect: MathlibEffect,
}

impl BoundaryReceiptSummary {
    /// Convert to a simulation vector for the dynamic visualization layer.
    pub fn to_simulation_vector(&self) -> serde_json::Value {
        serde_json::json!({
            "target": self.target,
            "strategy": self.strategy_class,
            "accepted": self.accepted,
            "novelty": self.novelty,
            "spend": self.genesis_margin.abs() as f64 / 1000.0, // Scaled for viz
            "defect": self.coherence_margin.abs() as f64 / 1000.0,
            "hash": self.receipt_hash,
            "failure_layer": self.failure_report.as_ref().map(|r| format!("{:?}", r.layer)),
            "failure_kind": self.failure_report.as_ref().map(|r| format!("{:?}", r.kind)),
        })
    }
}

/// Effect of mathlib on the proof attempt
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MathlibEffect {
    /// No mathlib used
    #[default]
    None,
    /// Strategy guidance only, no imports
    StrategyOnly,
    /// Imports helped the proof
    ImportHelped,
    /// Policy rejected the imports
    ImportRejected,
    /// Imports neither helped nor hurt
    ImportNeutral,
}


impl BoundaryReceiptSummary {
    /// Create a test receipt for unit testing
    #[cfg(test)]
    pub fn test_accepted(strategy_class: &str, novelty: f64) -> Self {
        Self {
            domain: "test".to_string(),
            target: "test_fn".to_string(),
            strategy_class: strategy_class.to_string(),
            wildness: 0.5,
            genesis_margin: 100,
            coherence_margin: 50,
            projection_defect: 0,
            tension_score: 0,
            provenance: "EXT".to_string(),
            record_tau: 0,
            semantic_distance: 0.0,
            accuracy: 1.0,
            utility: 1.0,
            sorry_detected: false,
            first_failure: String::new(),
            outcome: "accepted".to_string(),
            accepted: true,
            novelty,
            receipt_hash: "test_hash".to_string(),
            coh_template: None,
            mathlib_strategy: None,
            mathlib_confidence: None,
            mathlib_suggested_lemmas: None,
            mathlib_import_risk: None,
            mathlib_imports_used: false,
            mathlib_effect: MathlibEffect::None,
            failure_report: None,
        }
    }

    #[cfg(test)]
    pub fn test_rejected(strategy_class: &str, failure: &str) -> Self {
        Self {
            domain: "test".to_string(),
            target: "test_fn".to_string(),
            strategy_class: strategy_class.to_string(),
            wildness: 0.5,
            genesis_margin: -10,
            coherence_margin: -20,
            projection_defect: 10,
            tension_score: 50,
            provenance: "SIM".to_string(),
            record_tau: 0,
            semantic_distance: 0.0,
            accuracy: 5.0,
            utility: 0.1,
            sorry_detected: false,
            first_failure: failure.to_string(),
            outcome: "rejected".to_string(),
            accepted: false,
            novelty: 0.0,
            receipt_hash: "test_hash".to_string(),
            coh_template: None,
            mathlib_strategy: None,
            mathlib_confidence: None,
            mathlib_suggested_lemmas: None,
            mathlib_import_risk: None,
            mathlib_imports_used: false,
            mathlib_effect: MathlibEffect::None,
            failure_report: Some(crate::failure_taxonomy::FailureReport {
                candidate_id: "test".to_string(),
                target: "test_fn".to_string(),
                layer: crate::failure_taxonomy::FailureLayer::CohPost,
                kind: crate::failure_taxonomy::FailureKind::Governance(
                    crate::failure_taxonomy::GovernanceFailure::ProofCostTooHigh,
                ),
                raw_error: "test error".to_string(),
                normalized_message: "test error".to_string(),
                retryable: false,
                severity: crate::failure_taxonomy::FailureSeverity::HardInvalid,
                suggested_repairs: vec![],
                blocks_publication: true,
            }),
        }
    }
}

/// Public API functions
/// Initialize PhaseLoom state
pub fn phaseloom_init(config: &PhaseLoomConfig) -> PhaseLoomState {
    PhaseLoomState::new(config)
}

/// Ingest a boundary receipt
pub fn phaseloom_ingest(
    state: &mut PhaseLoomState,
    receipt: &BoundaryReceiptSummary,
    config: &PhaseLoomConfig,
) {
    state.ingest(receipt, config);
}

/// Sample next strategy (exploitation only)
pub fn phaseloom_sample(state: &PhaseLoomState) -> Option<String> {
    state.sample_best()
}

/// Sample next strategy using Boltzmann exploration
pub fn phaseloom_sample_boltzmann<R: rand::Rng>(
    state: &PhaseLoomState,
    config: &PhaseLoomConfig,
    rng: &mut R,
) -> (Option<String>, bool) {
    state.sample_strategy(config, rng)
}

/// Check circuit breaker
pub fn phaseloom_circuit_broken(state: &PhaseLoomState, config: &PhaseLoomConfig) -> bool {
    state.is_circuit_broken(config)
}

/// Serialize state for persistence
pub fn phaseloom_serialize(state: &PhaseLoomState) -> Result<Vec<u8>, serde_json::Error> {
    state.serialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phaseloom_init() {
        let config = PhaseLoomConfig::default();
        let state = PhaseLoomState::new(&config);

        assert_eq!(state.curvature, 0);
        assert_eq!(state.budget, config.initial_budget);
        assert_eq!(state.tau, 0);
        assert!(!state.circuit_broken);
    }

    #[test]
    fn test_ingest_accepted() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        let receipt = BoundaryReceiptSummary::test_accepted("synthesize", 0.8);
        state.ingest(&receipt, &config);

        assert_eq!(state.tau, 1);
        assert_eq!(state.accepted_count, 1);
        assert_eq!(state.curvature, 0); // No curvature increase on accept
        assert!(state.strategy_weights.0.contains_key("synthesize"));
    }

    #[test]
    fn test_ingest_rejected() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        let receipt = BoundaryReceiptSummary::test_rejected("synthesize", "policy_violation");
        state.ingest(&receipt, &config);

        assert_eq!(state.tau, 1);
        assert_eq!(state.rejected_count, 1);
        assert_eq!(state.curvature, 1); // Curvature increases on reject
    }

    #[test]
    fn test_weight_normalization() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        // Ingest multiple receipts
        state.ingest(
            &BoundaryReceiptSummary::test_accepted("synthesize", 0.5),
            &config,
        );
        state.ingest(
            &BoundaryReceiptSummary::test_accepted("refine", 0.5),
            &config,
        );
        state.ingest(
            &BoundaryReceiptSummary::test_rejected("debug", "error"),
            &config,
        );

        let sum: f64 = state.strategy_weights.0.values().sum();
        assert!((sum - 1.0).abs() < 0.001, "Weights should normalize to 1.0");
    }

    #[test]
    fn test_circuit_break() {
        let config = PhaseLoomConfig {
            circuit_break_threshold: 3,
            ..Default::default()
        };
        let mut state = PhaseLoomState::new(&config);

        // Reject enough to trigger circuit break (curvature = 4 > 3)
        for _ in 0..4 {
            state.ingest(
                &BoundaryReceiptSummary::test_rejected("test", "error"),
                &config,
            );
        }

        assert!(state.circuit_broken);
    }

    #[test]
    fn test_sample_strategy() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        // Add multiple strategies to have meaningful entropy
        state
            .strategy_weights
            .0
            .insert("synthesize".to_string(), 0.6);
        state.strategy_weights.0.insert("refine".to_string(), 0.3);
        state.strategy_weights.0.insert("debug".to_string(), 0.1);
        state.strategy_weights.normalize();

        let mut rng = rand::thread_rng();
        let (strategy, _was_exploration) = state.sample_strategy(&config, &mut rng);

        // Should get a valid strategy
        assert!(strategy.is_some());
        // Strategy should be one of the ones we added
        assert!(matches!(
            strategy.as_deref(),
            Some("synthesize") | Some("refine") | Some("debug")
        ));
    }

    #[test]
    fn test_compute_entropy() {
        let config = PhaseLoomConfig::default();
        let state = PhaseLoomState::new(&config);

        // Empty weights = zero entropy
        let entropy = state.compute_entropy();
        assert!((entropy - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_tracking() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        // Initially entropy should be 0
        assert!((state.current_entropy - 0.0).abs() < 0.001);

        // After adding a strategy, entropy gets recomputed on ingest
        state
            .strategy_weights
            .0
            .insert("synthesize".to_string(), 1.0);
        state.strategy_weights.normalize();
        state.current_entropy = state.compute_entropy();

        // Single strategy = low entropy
        let entropy = state.current_entropy;
        assert!(entropy < 0.5, "Single strategy should have low entropy");
    }

    #[test]
    fn test_temperature_decay() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        let initial_temp = state.temperature;

        // Decay multiple times
        for _ in 0..10 {
            state.update_temperature(&config);
        }

        // Temperature should have decayed
        assert!(state.temperature < initial_temp);

        // But should not go below min_temperature
        assert!(state.temperature >= config.min_temperature);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = PhaseLoomConfig::default();
        let state = PhaseLoomState::new(&config);

        let bytes = state.serialize().unwrap();
        let restored = PhaseLoomState::deserialize(&bytes).unwrap();

        assert_eq!(state.tau, restored.tau);
        assert_eq!(state.budget, restored.budget);
    }

    #[test]
    fn test_provenance_authority() {
        assert!(PhaseLoomState::provenance_authority("EXT") > PhaseLoomState::provenance_authority("DER"));
        assert!(PhaseLoomState::provenance_authority("DER") > PhaseLoomState::provenance_authority("REP"));
        assert!(PhaseLoomState::provenance_authority("REP") > PhaseLoomState::provenance_authority("SIM"));
    }

    #[test]
    fn test_calculate_read_cost() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);
        state.tau = 1000;

        // Same time, same provenance, same semantic = 0 cost (base case)
        let cost0 = state.calculate_read_cost(&config, 1000, 0.0, "EXT");
        assert_eq!(cost0, 0);

        // Older record = more cost
        let cost1 = state.calculate_read_cost(&config, 500, 0.0, "EXT");
        assert!(cost1 > 0);

        // Lower provenance authority = more cost (distance from EXT)
        let cost2 = state.calculate_read_cost(&config, 1000, 0.0, "SIM");
        assert!(cost2 > 0);
        
        // Semantic distance = more cost
        let cost3 = state.calculate_read_cost(&config, 1000, 10.0, "EXT");
        assert!(cost3 > 0);
    }

    #[test]
    fn test_anchor_firewall() {
        let config = PhaseLoomConfig::default();
        let state = PhaseLoomState::new(&config);

        // SIM cannot overwrite EXT
        assert!(!state.validate_memory_transition("SIM", "EXT"));
        // EXT can overwrite SIM
        assert!(state.validate_memory_transition("EXT", "SIM"));
        // DER can overwrite REP
        assert!(state.validate_memory_transition("DER", "REP"));
    }
}
