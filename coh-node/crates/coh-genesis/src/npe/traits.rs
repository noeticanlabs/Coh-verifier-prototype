

use super::engine::{NpeError, NpeProposal};

/// Trait for generating proposal content deterministically.
pub trait NpeGenerator {
    /// Context needed for generation (e.g., prompt, previous state, templates).
    type Context;

    /// Generate a single proposal's content deterministically based on seed and index.
    fn generate(&self, seed: u64, index: usize, ctx: &Self::Context) -> Result<String, NpeError>;
}

/// Trait for scoring proposals.
///
/// # Important Rule
/// Floating-point scores are *strictly advisory*. The final verification must use
/// integer/rational math via the Coh verifier.
pub trait NpeScorer {
    /// Score a proposal. Higher is better.
    fn score(&self, proposal: &NpeProposal) -> Result<f64, NpeError>;
}

/// Trait for verifying proposals via the Coherence boundary.
pub trait NpeVerifier {
    /// Verify a proposal. Returns a verdict string or receipt representation upon success,
    /// or an error if the verification process itself fails to execute.
    fn verify(&self, proposal: &NpeProposal) -> Result<String, NpeError>;
}
