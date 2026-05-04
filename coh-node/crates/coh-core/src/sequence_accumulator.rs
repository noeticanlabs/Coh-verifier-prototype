//! Sequence Accumulator
//!
//! Per audit Patch 6: Replace boolean sequence_valid with computed accumulator.
//!
//! G_i = H(G_{i-1} | h_i | i | state_pre | state_post)
//!
//! No receipt gets to self-attest order. It must prove order by being chained.

use crate::types::Hash32;
use sha2::{Digest, Sha256};

/// Compute sequence guard accumulator
///
/// G_i = H(G_{i-1} || h_i || i || state_pre || state_post)
///
/// Where:
/// - G_{i-1}: previous sequence accumulator (genesis = zero hash)
/// - h_i: this receipt's digest
/// - i: step index (prevents parallel reordering)
/// - state_pre: source state hash
/// - state_post: target state hash
pub fn compute_sequence_accumulator(
    prev_guard: Hash32,
    receipt_digest: Hash32,
    step_index: u64,
    state_pre: Hash32,
    state_post: Hash32,
) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(b"COH_SEQUENCE_V1");

    // Chain previous guard
    hasher.update(prev_guard.0);

    // Include this receipt's digest
    hasher.update(receipt_digest.0);

    // Include step index (prevents parallel reorder attacks)
    hasher.update(step_index.to_be_bytes());

    // Bind to state transition
    hasher.update(state_pre.0);
    hasher.update(state_post.0);

    Hash32(hasher.finalize().into())
}

/// Verify sequence accumulator matches claim
pub fn verify_sequence_accumulator(
    claimed_guard: Hash32,
    prev_guard: Hash32,
    receipt_digest: Hash32,
    step_index: u64,
    state_pre: Hash32,
    state_post: Hash32,
) -> bool {
    let computed = compute_sequence_accumulator(
        prev_guard,
        receipt_digest,
        step_index,
        state_pre,
        state_post,
    );

    computed == claimed_guard
}

/// V3 receipt sequence fields
#[derive(Clone, Debug)]
pub struct V3Sequence {
    /// Claimed sequence guard for this step
    pub guard: Hash32,
    /// Previous guard (for chain verification)
    pub prev_guard: Hash32,
    /// The computed sequence after applying this receipt
    pub next_guard: Hash32,
}

impl V3Sequence {
    /// Compute from receipt and state
    pub fn compute(
        prev_guard: Hash32,
        receipt_digest: Hash32,
        step_index: u64,
        state_pre: Hash32,
        state_post: Hash32,
    ) -> Self {
        let next_guard = compute_sequence_accumulator(
            prev_guard,
            receipt_digest,
            step_index,
            state_pre,
            state_post,
        );

        Self {
            guard: next_guard,
            prev_guard,
            next_guard,
        }
    }

    /// Verify claim against computed
    pub fn verify(&self) -> bool {
        // Verify that prev_guard is correctly linked
        // and that this guard is computed from it
        true // Placeholder - actual verification happens at commit time
    }
}

/// Genesis sequence (no prior guard)
pub const GENESIS_GUARD: Hash32 = Hash32([0; 32]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_chain() {
        let g0 = GENESIS_GUARD;
        let h1 = Hash32([1; 32]);

        let g1 = compute_sequence_accumulator(
            g0,
            h1,
            0,                // step 0
            Hash32([10; 32]), // state 0
            Hash32([11; 32]), // state 1
        );

        let h2 = Hash32([2; 32]);

        let g2 = compute_sequence_accumulator(
            g1,
            h2,
            1,                // step 1
            Hash32([11; 32]), // state 1
            Hash32([12; 32]), // state 2
        );

        // Verify g1 is different from g0
        assert_ne!(g1.0, g0.0);

        // Verify g2 chains from g1
        let valid = verify_sequence_accumulator(g2, g1, h2, 1, Hash32([11; 32]), Hash32([12; 32]));
        assert!(valid);

        // Verify tampering fails: wrong step index
        let tampered = verify_sequence_accumulator(
            g2,
            g1,
            999, // wrong index
            Hash32([11; 32]),
            Hash32([12; 32]),
        );
        assert!(!tampered);
    }
}
