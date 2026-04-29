//! Noetican Proposal Engine (NPE) Loop Implementation
//!
//! This module provides the core NPE loop: generate, mutate, score, verify, and refine.
//! It is designed to be used in `coh-genesis` for proposal search and refinement.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "npe-parallel")]
use rayon::prelude::*;

#[cfg(feature = "npe-graph")]
use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed};

/// The NPE loop error type
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NpeError {
    #[error("Seed error: {0}")]
    SeedError(String),

    #[error("Generation error: {0}")]
    GenerationError(String),

    #[error("Scoring error: {0}")]
    ScoringError(String),

    #[error("Verification error: {0}")]
    VerificationError(String),

    #[error("Graph error: {0}")]
    GraphError(String),
}

/// The NPE proposal state
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NpeProposal {
    /// Unique identifier for this proposal
    pub id: String,
    /// The semantic content (Lean code, math expression, etc.)
    pub content: String,
    /// The generation seed used
    pub seed: u64,
    /// Current score (advisory, not final)
    pub score: f64,
    /// Proposal hash for dedup
    pub content_hash: String,
    /// Mutation depth from root
    pub depth: u32,
    /// Parent proposal ID (if any)
    pub parent_id: Option<String>,
    /// [ECOLOGY] Creation time in PhaseLoom tau
    pub tau: u64,
    /// [ECOLOGY] Epistemic provenance
    pub provenance: String,
    /// Status
    pub status: ProposalStatus,
}

/// Proposal status in the NPE loop
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProposalStatus {
    /// Generated, pending scoring
    #[default]
    Generated,
    /// Scored, pending verification
    Scored,
    /// Sent to verifier
    Verifying,
    /// Accepted by verifier
    Accepted,
    /// Rejected by verifier
    Rejected(String),
    /// Failed in generation
    Failed(String),
}

/// Edge type for NPE proposal graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpeEdge {
    /// Mutation type (e.g., "mutate", "simplify", "rewrite")
    pub mutation_type: String,
    /// Score delta (advisory)
    pub score_delta: f64,
    /// Verdict from verifier (if any)
    pub verdict: Option<String>,
}

/// NPE proposal graph using petgraph
#[cfg(feature = "npe-graph")]
pub struct NpeProposalGraph {
    /// The underlying graph
    graph: StableGraph<NpeProposal, NpeEdge, Directed>,
    /// Map from proposal ID to node index
    id_to_index: std::collections::HashMap<String, NodeIndex>,
    /// Root proposal ID (if any)
    root_id: Option<String>,
}

#[cfg(feature = "npe-graph")]
impl NpeProposalGraph {
    /// Create a new proposal graph
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
            id_to_index: std::collections::HashMap::new(),
            root_id: None,
        }
    }

    /// Add a proposal to the graph
    pub fn add_proposal(&mut self, proposal: NpeProposal, parent_id: Option<String>) -> NodeIndex {
        // Add node to graph
        let index = self.graph.add_node(proposal.clone());

        // Update ID map
        self.id_to_index.insert(proposal.id.clone(), index);

        // If there's a parent, add edge
        if let Some(pid) = parent_id {
            if let Some(parent_index) = self.id_to_index.get(&pid) {
                let edge = NpeEdge {
                    mutation_type: "mutate".to_string(),
                    score_delta: 0.0,
                    verdict: None,
                };
                self.graph.add_edge(*parent_index, index, edge);
            }
        }

        // If this is the first node, set as root
        if self.root_id.is_none() {
            self.root_id = Some(proposal.id.clone());
        }

        index
    }

    /// Add a proposal with explicit edge data
    pub fn add_proposal_with_edge(
        &mut self,
        proposal: NpeProposal,
        parent_id: Option<String>,
        edge: NpeEdge,
    ) -> NodeIndex {
        let index = self.graph.add_node(proposal.clone());
        self.id_to_index.insert(proposal.id.clone(), index);

        if let Some(pid) = parent_id {
            if let Some(parent_index) = self.id_to_index.get(&pid) {
                self.graph.add_edge(*parent_index, index, edge);
            }
        }

        if self.root_id.is_none() {
            self.root_id = Some(proposal.id.clone());
        }

        index
    }

    /// Get a proposal by ID
    pub fn get_proposal(&self, id: &str) -> Option<&NpeProposal> {
        self.id_to_index
            .get(id)
            .and_then(|idx| self.graph.node_weight(*idx))
    }

    /// Get all accepted proposals
    pub fn accepted_proposals(&self) -> Vec<&NpeProposal> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let p = self.graph.node_weight(idx)?;
                if p.status == ProposalStatus::Accepted {
                    Some(p)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all rejected proposals
    pub fn rejected_proposals(&self) -> Vec<&NpeProposal> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let p = self.graph.node_weight(idx)?;
                if matches!(p.status, ProposalStatus::Rejected(_)) {
                    Some(p)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Get the root ID
    pub fn root_id(&self) -> Option<&str> {
        self.root_id.as_deref()
    }
}

#[cfg(feature = "npe-graph")]
impl Default for NpeProposalGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// NPE loop configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpeConfig {
    /// Seed for deterministic generation
    pub seed: u64,
    /// Maximum proposals to keep in beam
    pub beam_width: usize,
    /// Maximum mutation depth
    pub max_depth: u32,
    /// Number of parallel candidates to generate
    pub batch_size: usize,
    /// Enable parallel scoring (requires rayon)
    #[cfg(feature = "npe-parallel")]
    pub parallel_scoring: bool,
}

impl Default for NpeConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            beam_width: 10,
            max_depth: 5,
            batch_size: 100,
            #[cfg(feature = "npe-parallel")]
            parallel_scoring: true,
        }
    }
}

/// NPE loop state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpeState {
    /// Current configuration
    pub config: NpeConfig,
    /// Current proposals
    pub proposals: Vec<NpeProposal>,
    /// Accepted proposals
    pub accepted: Vec<NpeProposal>,
    /// Rejected proposals with reasons
    pub rejected: Vec<NpeProposal>,
    /// Generation counter
    pub generation: u64,
    /// Last accepted score
    pub best_score: f64,
}

impl NpeState {
    pub fn new(config: NpeConfig) -> Self {
        Self {
            config,
            proposals: Vec::new(),
            accepted: Vec::new(),
            rejected: Vec::new(),
            generation: 0,
            best_score: f64::NEG_INFINITY,
        }
    }

    /// Reset the state
    pub fn reset(&mut self) {
        self.proposals.clear();
        self.accepted.clear();
        self.rejected.clear();
        self.generation = 0;
        self.best_score = f64::NEG_INFINITY;
    }

    /// Add a proposal to the state
    pub fn add_proposal(&mut self, proposal: NpeProposal) {
        if proposal.score > self.best_score {
            self.best_score = proposal.score;
        }
        self.proposals.push(proposal);
    }
}

/// NPE Engine for deterministic proposal generation and search
pub struct NpeEngine {
    config: NpeConfig,
}

impl NpeEngine {
    /// Create a new NPE engine with the given configuration
    pub fn new(config: NpeConfig) -> Self {
        Self { config }
    }

    /// Create a new NPE engine with default configuration
    pub fn new_default() -> Self {
        Self {
            config: NpeConfig::default(),
        }
    }

    /// Generate proposals with a given seed.
    /// The generator is deterministic: same seed + same config => same proposals.
    pub fn generate_proposals<G: crate::npe::traits::NpeGenerator>(
        &self,
        count: usize,
        generator: &G,
        ctx: &G::Context,
    ) -> Result<Vec<NpeProposal>, NpeError> {
        use sha2::{Digest, Sha256};

        let mut proposals = Vec::with_capacity(count);
        for i in 0..count {
            let id = format!("p-{:08}-{:04}", self.config.seed, i);
            let content = generator.generate(self.config.seed, i, ctx)?;

            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let content_hash = hex::encode(hasher.finalize());

            proposals.push(NpeProposal {
                id,
                content,
                seed: self.config.seed,
                score: 0.0,
                content_hash,
                depth: 0,
                parent_id: None,
                tau: 0,                        // Default to 0, should be set by ingest or loop
                provenance: "SIM".to_string(), // Default for newly generated
                status: ProposalStatus::Generated,
            });
        }

        Ok(proposals)
    }

    /// Score proposals (advisory only - this ranking is advisory)
    ///
    /// # Important Rule
    ///
    /// Floating-point scores are *strictly advisory*. The final verification must use integer/rational math
    /// via the Coh verifier. This function only ranks proposals for selection.
    pub fn score_proposals<S: crate::npe::traits::NpeScorer>(
        &self,
        proposals: &mut [NpeProposal],
        scorer: &S,
    ) -> Result<(), NpeError> {
        for p in proposals.iter_mut() {
            p.score = scorer.score(p)?;
        }

        // Sort by score (higher is better)
        proposals.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(())
    }

    /// Apply proposals to state
    pub fn apply_to_state(&self, state: &mut NpeState, proposals: Vec<NpeProposal>) {
        state.generation += 1;
        for proposal in proposals {
            state.add_proposal(proposal);
        }
    }

    /// Get the beam (top proposals by score)
    pub fn get_beam<'a>(&self, state: &'a NpeState) -> Vec<&'a NpeProposal> {
        state
            .proposals
            .iter()
            .take(self.config.beam_width)
            .collect()
    }
}

/// Parallel batch scoring using rayon
#[cfg(feature = "npe-parallel")]
pub fn parallel_score_proposals<S: crate::npe::traits::NpeScorer + Sync>(
    proposals: &mut [NpeProposal],
    scorer: &S,
) -> Result<(), NpeError> {
    use std::sync::Mutex;

    // Score in parallel, capturing any first error
    let err = Mutex::new(None);

    proposals
        .par_iter_mut()
        .for_each(|p| match scorer.score(p) {
            Ok(score) => p.score = score,
            Err(e) => {
                let mut lock = err.lock().unwrap();
                if lock.is_none() {
                    *lock = Some(e);
                }
            }
        });

    if let Some(e) = err.into_inner().unwrap() {
        return Err(e);
    }

    // Sort canonically after parallel scoring
    proposals.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(())
}

/// Test deterministic replay
///
/// Lemma 1: If the proposal generator uses a fixed seed and all receipts are canonically serialized,
/// then the same input ledger produces the same proposal sequence.
///
/// H_0, seed => (p_1, ..., p_n)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_replay() {
        struct DummyGen;
        impl crate::npe::traits::NpeGenerator for DummyGen {
            type Context = ();
            fn generate(&self, seed: u64, index: usize, _ctx: &()) -> Result<String, NpeError> {
                Ok(format!("test-{}-{}", seed, index))
            }
        }

        let config = NpeConfig {
            seed: 12345,
            beam_width: 10,
            max_depth: 3,
            batch_size: 100,
            #[cfg(feature = "npe-parallel")]
            parallel_scoring: false,
        };

        let engine = NpeEngine::new(config.clone());
        let gen = DummyGen;

        // Generate twice with same seed
        let proposals1 = engine.generate_proposals(10, &gen, &()).unwrap();
        let proposals2 = engine.generate_proposals(10, &gen, &()).unwrap();

        // Should be identical
        assert_eq!(proposals1.len(), proposals2.len());
        for (p1, p2) in proposals1.iter().zip(proposals2.iter()) {
            assert_eq!(p1.id, p2.id);
            assert_eq!(p1.seed, p2.seed);
            assert_eq!(p1.score, p2.score);
        }
    }

    #[test]
    fn test_proposal_state() {
        let config = NpeConfig::default();
        let engine = NpeEngine::new(config.clone());
        let mut state = NpeState::new(config);

        let proposals = vec![
            NpeProposal {
                id: "p1".to_string(),
                content: "test 1".to_string(),
                seed: 42,
                score: 0.5,
                content_hash: "hash1".to_string(),
                depth: 0,
                parent_id: None,
                tau: 0,
                provenance: "SIM".to_string(),
                status: ProposalStatus::Generated,
            },
            NpeProposal {
                id: "p2".to_string(),
                content: "test 2".to_string(),
                seed: 42,
                score: 0.8,
                content_hash: "hash2".to_string(),
                depth: 0,
                parent_id: None,
                tau: 0,
                provenance: "SIM".to_string(),
                status: ProposalStatus::Generated,
            },
        ];

        engine.apply_to_state(&mut state, proposals);

        assert_eq!(state.generation, 1);
        assert_eq!(state.proposals.len(), 2);
        assert_eq!(state.best_score, 0.8);
    }

    #[cfg(feature = "npe-graph")]
    #[test]
    fn test_proposal_graph() {
        let mut graph = NpeProposalGraph::new();

        // Add root
        let root = NpeProposal {
            id: "root".to_string(),
            content: "root content".to_string(),
            seed: 42,
            score: 0.5,
            content_hash: "hash1".to_string(),
            depth: 0,
            parent_id: None,
            status: ProposalStatus::Generated,
            ..Default::default()
        };
        graph.add_proposal(root, None);

        // Add child
        let child = NpeProposal {
            id: "child".to_string(),
            content: "child content".to_string(),
            seed: 43,
            score: 0.8,
            content_hash: "hash2".to_string(),
            depth: 1,
            parent_id: Some("root".to_string()),
            status: ProposalStatus::Generated,
            ..Default::default()
        };
        graph.add_proposal(child, Some("root".to_string()));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.root_id(), Some("root"));
    }
}
