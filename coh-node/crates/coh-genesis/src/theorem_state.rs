//! Theorem State Commitment
//!
//! Provides canonical commitment to theorem state using filesystem snapshots.
//!
//! This replaces synthetic data like "pre_TestTheorem" with real state commitments:
//!   theorem_hash_pre = H(pre-repair theorem source)
//!   theorem_hash_post = H(post-repair theorem source + proof_hash)
//!
//! Chain inputs come from previous accepted CohBit trajectory.

use coh_npe::tools::CtrlRepairReceipt;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Domain tag for theorem state commitments
const DOMAIN_TAG: &[u8] = b"COH_THEOREM_STATE_V1";

/// Compute canonical theorem state commitment from source text
///
/// This binds: theorem name + file path + source text + optional proof hash
pub fn compute_theorem_state_commitment(
    theorem_name: &str,
    file_path: &Path,
    source_text: &str,
    proof_hash: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();

    // Domain separation
    hasher.update(DOMAIN_TAG);

    // Theorem identity
    hasher.update(theorem_name.as_bytes());

    // Canonical file path
    let canonical_path = file_path
        .canonicalize()
        .unwrap_or_else(|_| file_path.to_path_buf());
    hasher.update(canonical_path.to_string_lossy().as_bytes());

    // Source snapshot (the authoritative state)
    hasher.update(source_text.as_bytes());

    // Include proof hash if available (proves verified post-state)
    if let Some(proof) = proof_hash {
        hasher.update(b"LEAN_PROOF_HASH");
        hasher.update(proof.as_bytes());
    }

    // Return hex encoding
    hex::encode(hasher.finalize())
}

/// Capture pre-repair theorem state from the filesystem
pub fn capture_pre_state(theorem_name: &str, theorem_file: &Path) -> Result<String, String> {
    let source = std::fs::read_to_string(theorem_file)
        .map_err(|e| format!("failed to read theorem source: {}", e))?;

    Ok(compute_theorem_state_commitment(
        theorem_name,
        theorem_file,
        &source,
        None, // No proof yet
    ))
}

/// Capture post-repair theorem state from verified source
pub fn capture_post_state(
    theorem_name: &str,
    theorem_file: &Path,
    proof_hash: &str,
) -> Result<String, String> {
    let source = std::fs::read_to_string(theorem_file)
        .map_err(|e| format!("failed to read theorem source: {}", e))?;

    Ok(compute_theorem_state_commitment(
        theorem_name,
        theorem_file,
        &source,
        Some(proof_hash),
    ))
}

/// Compute proof-only hash (for Lean-specific witness)
pub fn compute_proof_hash(lean_response: &serde_json::Value) -> String {
    let proof = lean_response
        .get("proof_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("no_proof");

    let mut hasher = Sha256::new();
    hasher.update(b"LEAN_PROOF_V1");
    hasher.update(proof.as_bytes());
    hex::encode(hasher.finalize())
}

/// Derive chain inputs from trajectory
///
/// Uses previous accepted receipt to compute:
///   - previous sequence accumulator
///   - previous chain digest
pub fn derive_chain_inputs(receipts: &[CtrlRepairReceipt]) -> (Option<String>, Option<String>) {
    if receipts.is_empty() {
        // Genesis case - no previous
        return (None, None);
    }

    let last = receipts.last().unwrap();

    // Previous sequence accumulator
    let prev_seq = if receipts.len() == 1 {
        // First receipt uses genesis
        None
    } else {
        Some(last.sequence_accumulator.clone())
    };

    // Previous chain digest = last receipt's post-state hash
    let prev_chain = Some(last.theorem_hash_post.clone());

    (prev_seq, prev_chain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_deterministic() {
        let path = Path::new("/test.lean");
        let source = "theorem test : 1 + 1 = 2 := by ring";

        let hash1 = compute_theorem_state_commitment("test", path, source, Some("0xabc"));
        let hash2 = compute_theorem_state_commitment("test", path, source, Some("0xabc"));

        assert_eq!(hash1, hash2, "same input should produce same hash");
    }

    #[test]
    fn test_commitment_changes_with_source() {
        let path = Path::new("/test.lean");
        let pre = "theorem test : 1 + 1 = 2 := by ring";
        let post = "theorem test : 1 + 1 = 2 := by linarith";

        let pre_hash = compute_theorem_state_commitment("test", path, pre, None);
        let post_hash = compute_theorem_state_commitment("test", path, post, Some("0xdef"));

        assert_ne!(
            pre_hash, post_hash,
            "source change should produce different hash"
        );
    }

    #[test]
    fn test_proof_hash() {
        let json = serde_json::json!({
            "result": "success",
            "proof_hash": "0xdef456"
        });

        let proof = compute_proof_hash(&json);
        // Hex encoding produces 64 char hash, not "0x" prefix
        assert!(proof.len() == 64, "should produce 64-char hex hash");
    }
}
