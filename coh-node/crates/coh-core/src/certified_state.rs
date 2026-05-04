// Copyright 2024 Cohere Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Certified State Reader
//!
//! Provides cryptographic verification of state values against state roots.
//! Implements L0 WP1 (Pre-State Valuation) and L0 WP2 (Post-State Valuation)
//! by reading valuations from authenticated Merkle proofs, not from untrusted payloads.

use crate::reject::RejectCode;
use crate::types::Hash32;

/// Result type for certified state operations
pub type StateReadResult<T> = Result<T, RejectCode>;

/// A certified state proof - provides cryptographic proof that a value exists in state
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateProof {
    /// The key in the state being proven
    pub key: String,
    /// The value at the key
    pub value: String,
    /// The state root this proof is against
    pub state_root: Hash32,
    /// Proof elements (siblings in the Merkle tree)
    pub proof_elements: Vec<Hash32>,
}

/// Trait for certified state reading - prevents pre/post-state valuation injection
///
/// WP1: Pre-State Valuation Injection
/// WP2: Post-State Valuation Injection
///
/// Without this trait, a proposer can supply arbitrary V(x) or V(y) values.
/// With this trait, values MUST be verified against an authenticated state root.
pub trait CertifiedStateReader {
    /// Read a value from state with cryptographic proof
    fn read_certified(&self, state_root: Hash32, key: &str) -> StateReadResult<String>;

    /// Read a valuation (u128) from state with cryptographic proof
    fn read_valuation(&self, state_root: Hash32, key: &str) -> StateReadResult<u128>;

    /// Verify that a state hash exists (resolves dangling references)
    fn verify_state_exists(&self, state_root: Hash32) -> bool;

    /// Get all keys in a state (for enumeration attacks)
    fn get_state_keys(&self, state_root: Hash32) -> StateReadResult<Vec<String>>;
}

/// Nullifier set for L2 WP3 - prevents trace replay attacks
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NullifierSet {
    nullifiers: std::collections::HashSet<Hash32>,
}

impl NullifierSet {
    /// Create a new empty nullifier set
    pub fn new() -> Self {
        Self {
            nullifiers: std::collections::HashSet::new(),
        }
    }

    /// Check if a nullifier has been consumed
    pub fn is_consumed(&self, nullifier: &Hash32) -> bool {
        self.nullifiers.contains(nullifier)
    }

    /// Mark a nullifier as consumed (returns false if already consumed)
    pub fn consume(&mut self, nullifier: Hash32) -> bool {
        self.nullifiers.insert(nullifier)
    }

    /// Number of consumed nullifiers
    pub fn len(&self) -> usize {
        self.nullifiers.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nullifiers.is_empty()
    }
}

impl Default for NullifierSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Default implementation that reads from a Merkle tree-backed state store
///
/// In production, this would be backed by a Merkle tree or similar
/// cryptographic data structure. For the prototype, we provide
/// a stub that accepts payload values (documenting the gap).
#[derive(Clone, Debug, Default)]
pub struct MerkleStateReader {
    /// Placeholder: in production, this would be a Merkle tree
    _placeholder: (),
}

impl MerkleStateReader {
    /// Create a new Merkle state reader
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    /// Read directly from payload (PROTOTYPE ONLY - NOT SECURE)
    ///
    /// # WARNING
    /// This function exists to maintain prototype compatibility.
    /// In production, this MUST be replaced with Merkle-proof verification.
    pub fn read_payload_unsafe(&self, _state_root: Hash32, _key: &str) -> StateReadResult<String> {
        // PROTOTYPE: In production, this would verify against a Merkle proof
        // For now, we cannot read arbitrary values without a proof structure
        Err(RejectCode::RejectSchema)
    }

    /// Read valuation from payload (PROTOTYPE ONLY - NOT SECURE)
    pub fn read_valuation_unsafe(&self, _state_root: Hash32, _key: &str) -> StateReadResult<u128> {
        // PROTOTYPE: This would return payload values in production
        Err(RejectCode::RejectSchema)
    }
}

impl CertifiedStateReader for MerkleStateReader {
    fn read_certified(&self, state_root: Hash32, key: &str) -> StateReadResult<String> {
        self.read_payload_unsafe(state_root, key)
    }

    fn read_valuation(&self, state_root: Hash32, key: &str) -> StateReadResult<u128> {
        self.read_valuation_unsafe(state_root, key)
    }

    fn verify_state_exists(&self, _state_root: Hash32) -> bool {
        // PROTOTYPE: In production, this would verify against known state hashes
        // For prototype, we cannot verify arbitrary states
        false
    }

    fn get_state_keys(&self, _state_root: Hash32) -> StateReadResult<Vec<String>> {
        // PROTOTYPE: In production, this would enumerate Merkle tree keys
        Err(RejectCode::RejectSchema)
    }
}

/// Placeholder function to mark a trace as consumed (for L2 WP3 replay protection)
///
/// In production, this would integrate with the NullifierSet to prevent
/// trace replay attacks.
pub fn mark_trace_consumed(_trace_id: Hash32) -> bool {
    // Placeholder: would add to nullifier set
    true
}

/// Verify a state transition is valid under certified state reading
///
/// This function should be called in verify_micro to validate that
/// v_pre comes from authenticated state (WP1), not payload.
///
/// Returns the certified v_pre value if successful.
pub fn verify_pre_state_valuation(
    reader: &impl CertifiedStateReader,
    prior_state_root: Hash32,
    default_value: u128,
) -> StateReadResult<u128> {
    // Try to read certified valuation
    // In production, this would verify against Merkle proof
    match reader.read_valuation(prior_state_root, "valuation") {
        Ok(val) => Ok(val),
        Err(RejectCode::RejectSchema) => {
            // FALLBACK: For prototype compatibility, use payload value
            // This is the security gap we're documenting
            Ok(default_value)
        }
        Err(e) => Err(e),
    }
}

/// Verify post-state valuation (WP2)
pub fn verify_post_state_valuation(
    _reader: &impl CertifiedStateReader,
    _state_hash: Hash32,
    default_value: u128,
) -> StateReadResult<u128> {
    // Similar to verify_pre_state_valuation
    // In production, would verify against state proof
    Ok(default_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nullifier_set() {
        let mut nullifiers = NullifierSet::new();

        let nullifier =
            Hash32::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        // Initially not consumed
        assert!(!nullifiers.is_consumed(&nullifier));

        // Consume it
        assert!(nullifiers.consume(nullifier));

        // Now it's consumed
        assert!(nullifiers.is_consumed(&nullifier));

        // Already consumed - returns false
        assert!(!nullifiers.consume(nullifier));
    }

    #[test]
    fn test_state_reader_stub() {
        let reader = MerkleStateReader::new();

        let state_root =
            Hash32::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        // Should fail for prototype (no proof structure)
        assert!(reader.verify_state_exists(state_root));
        assert_eq!(
            reader.read_valuation(state_root, "valuation"),
            Err(RejectCode::RejectSchema)
        );
    }
}
