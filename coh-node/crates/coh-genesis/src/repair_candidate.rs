use serde::{Deserialize, Serialize};

/// Strategy source for repair candidate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairSource {
    LemmaForge,
    EquivalenceHunter,
    InvariantHunter,
    Fallback,
    Generated,
}

/// Repair strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairStrategy {
    Exact,
    Rfl,
    Simp,
    SimpAll,
    Omega,
    Ring,
    Linarith,
    Aesop,
    LocalLemma,
    GiveUp,
}

impl RepairStrategy {
    pub fn confidence(&self) -> f32 {
        match self {
            RepairStrategy::Exact => 1.0,
            RepairStrategy::Rfl => 0.95,
            RepairStrategy::Simp => 0.8,
            RepairStrategy::SimpAll => 0.7,
            RepairStrategy::Omega => 0.75,
            RepairStrategy::Ring => 0.75,
            RepairStrategy::Linarith => 0.7,
            RepairStrategy::Aesop => 0.6,
            RepairStrategy::LocalLemma => 0.5,
            RepairStrategy::GiveUp => 0.0,
        }
    }

    pub fn rank(&self) -> u8 {
        match self {
            RepairStrategy::Exact => 1,
            RepairStrategy::Rfl => 2,
            RepairStrategy::Simp => 3,
            RepairStrategy::SimpAll => 4,
            RepairStrategy::Omega => 5,
            RepairStrategy::Ring => 6,
            RepairStrategy::Linarith => 7,
            RepairStrategy::Aesop => 8,
            RepairStrategy::LocalLemma => 9,
            RepairStrategy::GiveUp => 99,
        }
    }
}

impl From<&str> for RepairStrategy {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "exact" => RepairStrategy::Exact,
            "rfl" => RepairStrategy::Rfl,
            "simp" => RepairStrategy::Simp,
            "omega" => RepairStrategy::Omega,
            "ring" => RepairStrategy::Ring,
            "linarith" => RepairStrategy::Linarith,
            "aesop" => RepairStrategy::Aesop,
            _ => RepairStrategy::GiveUp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairCandidate {
    pub tactic: String,
    pub strategy: RepairStrategy,
    pub confidence: f32,
    pub source: RepairSource,
    pub proof_text: Option<String>,
}

impl RepairCandidate {
    pub fn new(tactic: String, strategy: RepairStrategy, source: RepairSource) -> Self {
        Self {
            confidence: strategy.confidence(),
            tactic,
            strategy,
            source,
            proof_text: None,
        }
    }

    pub fn with_proof(mut self, proof: String) -> Self {
        self.proof_text = Some(proof);
        self
    }
}

/// Collection of candidates that sorts by confidence/rank
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CandidatePool {
    pub candidates: Vec<RepairCandidate>,
}

impl CandidatePool {
    pub fn new() -> Self {
        Self { candidates: vec![] }
    }

    pub fn add(&mut self, candidate: RepairCandidate) {
        self.candidates.push(candidate);
    }

    /// Get candidates sorted by confidence (best first)
    pub fn ranked(&self) -> Vec<RepairCandidate> {
        let mut sorted = self.candidates.clone();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.strategy.rank().cmp(&b.strategy.rank()))
        });
        sorted
    }

    /// Get best candidate (highest confidence)
    pub fn best(&self) -> Option<RepairCandidate> {
        self.ranked().into_iter().next()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }
}
