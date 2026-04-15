//! APE Proposal Data Structures
//!
//! Defines the structured candidates for Coh Wedge verification.

use coh_core::types::{MicroReceiptWire, SlabReceiptWire};
use serde::{Deserialize, Serialize};

/// Strategy used to generate the proposal
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    /// Slightly corrupt valid states
    Mutation,
    /// Merge multiple states incorrectly
    Recombination,
    /// Break invariants  
    Violation,
    /// Stress numeric boundaries
    Overflow,
    /// Break logical coherence
    Contradiction,
}

impl Strategy {
    pub fn name(&self) -> &'static str {
        match self {
            Strategy::Mutation => "mutation",
            Strategy::Recombination => "recombination",
            Strategy::Violation => "violation",
            Strategy::Overflow => "overflow",
            Strategy::Contradiction => "contradiction",
        }
    }

    /// Human-readable explanation of what this strategy does
    pub fn note(&self) -> &'static str {
        match self {
            Strategy::Mutation => "altered receipt field while preserving surface structure",
            Strategy::Recombination => "spliced valid fragments into invalid chain topology",
            Strategy::Violation => "broke invariant directly",
            Strategy::Overflow => "exceeded bounds or numeric domain assumptions",
            Strategy::Contradiction => "created mutually incompatible claims in one proposal",
        }
    }

    /// Generate a candidate using this strategy
    /// Note: For now, returns raw Candidate. The metadata is added at the call site.
    pub fn generate(&self, input: &Input, rng: &mut crate::seed::SeededRng) -> Candidate {
        use crate::strategies::{contradiction, mutation, overflow, recombination, violation};
        match self {
            Strategy::Mutation => mutation::run(input, rng),
            Strategy::Recombination => recombination::run(input, rng),
            Strategy::Violation => violation::run(input, rng),
            Strategy::Overflow => overflow::run(input, rng),
            Strategy::Contradiction => contradiction::run(input, rng),
        }
    }

    /// Get all strategy variants
    pub fn all() -> [Strategy; 5] {
        [
            Strategy::Mutation,
            Strategy::Recombination,
            Strategy::Violation,
            Strategy::Overflow,
            Strategy::Contradiction,
        ]
    }
}

/// Input to the proposal engine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    /// Optional base receipt to mutate
    pub base_micro: Option<MicroReceiptWire>,
    /// Optional base chain to mutate
    pub base_chain: Option<Vec<MicroReceiptWire>>,
    /// Optional base slab to mutate  
    pub base_slab: Option<SlabReceiptWire>,
    /// Prompt describes what behavior we want from LLM simulation
    pub prompt: String,
}

impl Input {
    /// Create input from single receipt
    pub fn from_micro(receipt: MicroReceiptWire) -> Self {
        Self {
            base_micro: Some(receipt),
            base_chain: None,
            base_slab: None,
            prompt: String::new(),
        }
    }

    /// Create input from chain
    pub fn from_chain(chain: Vec<MicroReceiptWire>) -> Self {
        Self {
            base_micro: None,
            base_chain: Some(chain),
            base_slab: None,
            prompt: String::new(),
        }
    }

    /// Create input from slab
    pub fn from_slab(slab: SlabReceiptWire) -> Self {
        Self {
            base_micro: None,
            base_chain: None,
            base_slab: Some(slab),
            prompt: String::new(),
        }
    }

    /// Empty input for pure generation
    pub fn empty() -> Self {
        Self {
            base_micro: None,
            base_chain: None,
            base_slab: None,
            prompt: String::new(),
        }
    }
}

/// Generated proposal candidate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    /// Description of what was done
    pub prompt: String,
    /// Unique ID (generated hash)
    pub proposal_id: String,
    /// Strategy used
    pub strategy: Strategy,
    /// Seed used for generation
    pub seed: u64,
    /// The candidate data
    pub candidate: Candidate,
}

impl Proposal {
    /// Create new proposal
    pub fn new(strategy: Strategy, seed: u64, candidate: Candidate) -> Self {
        let proposal_id = format!(
            "{:016x}-{:x}",
            seed,
            candidate.content_hash() & 0xFFFFFFFFFFFFF
        );

        Self {
            prompt: format!("{}: seed={}", strategy.name(), seed),
            proposal_id,
            strategy,
            seed,
            candidate,
        }
    }
}

/// Candidate type with data
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Candidate {
    /// Single micro receipt
    Micro(MicroReceiptWire),
    /// Chain of receipts
    Chain(Vec<MicroReceiptWire>),
    /// Slab receipt
    Slab(SlabReceiptWire),
}

/// Metadata for candidate (for replayability and explainability)
/// Note: Not serialized - used at runtime for demo output
pub struct CandidateMetadata {
    /// Strategy that generated this candidate
    pub strategy_name: &'static str,
    /// Specific attack type within the strategy
    pub attack_kind: &'static str,
    /// Seed used for generation (for replay)
    pub seed: u64,
    /// Human-readable explanation of the corruption
    pub notes: String,
}

impl CandidateMetadata {
    pub fn new(
        strategy_name: &'static str,
        attack_kind: &'static str,
        seed: u64,
        notes: String,
    ) -> Self {
        Self {
            strategy_name,
            attack_kind,
            seed,
            notes,
        }
    }
}

impl Candidate {
    /// Get the internal content hash (for ID generation)
    ///
    /// Not a cryptographic hash - just for uniqueness
    pub fn content_hash(&self) -> u64 {
        use serde_json::to_string;
        match self {
            Candidate::Micro(w) => {
                let s = to_string(w).unwrap_or_default();
                s.len() as u64
            }
            Candidate::Chain(v) => {
                let s = to_string(v).unwrap_or_default();
                s.len() as u64
            }
            Candidate::Slab(w) => {
                let s = to_string(w).unwrap_or_default();
                s.len() as u64
            }
        }
    }

    /// Get micro receipt if present
    pub fn as_micro(&self) -> Option<&MicroReceiptWire> {
        match self {
            Candidate::Micro(w) => Some(w),
            _ => None,
        }
    }

    /// Get chain if present
    pub fn as_chain(&self) -> Option<&Vec<MicroReceiptWire>> {
        match self {
            Candidate::Chain(v) => Some(v),
            _ => None,
        }
    }

    /// Get slab if present
    pub fn as_slab(&self) -> Option<&SlabReceiptWire> {
        match self {
            Candidate::Slab(w) => Some(w),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coh_core::types::MetricsWire;

    fn sample_micro() -> MicroReceiptWire {
        MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "test".to_string(),
            canon_profile_hash: "0".repeat(64),
            policy_hash: "0".repeat(64),
            step_index: 0,
            step_type: None,
            signatures: None,
            state_hash_prev: "0".repeat(64),
            state_hash_next: "0".repeat(64),
            chain_digest_prev: "0".repeat(64),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: "100".to_string(),
                v_post: "80".to_string(),
                spend: "15".to_string(),
                defect: "0".to_string(),
            },
        }
    }

    #[test]
    fn test_proposal_new() {
        let micro = sample_micro();
        let candidate = Candidate::Micro(micro);
        let proposal = Proposal::new(Strategy::Mutation, 42, candidate);

        assert_eq!(proposal.seed, 42);
        assert_eq!(proposal.strategy, Strategy::Mutation);
    }

    #[test]
    fn test_input_from_micro() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        assert!(input.base_micro.is_some());
        assert!(input.base_chain.is_none());
    }
}
