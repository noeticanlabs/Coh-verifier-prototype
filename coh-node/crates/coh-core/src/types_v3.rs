//! V3 Types - Transition Contract extensions
//!
//! Extends V1/V2 types with:
//! - Objective layer (objective_result, optional)
//! - Sequence guard (sequence_valid)
//! - Policy governance (override_applied)

use crate::reject::RejectCode;
use crate::types::{Hash32, Metrics};
use serde::{Deserialize, Serialize};

/// Objective target types for V3
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveTarget {
    #[default]
    MinimizeSpend,
    MaximizeValue,
    CloseTickets,
    ZeroPending,
    Custom(String),
}

/// Objective result if checked
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveResult {
    Satisfied(ObjectiveTarget),
    Violated(ObjectiveTarget),
    #[default]
    NotApplicable,
}

/// V3 MicroReceipt - extends V1/V2 with Transition Contract fields
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MicroReceiptV3Wire {
    // Base V2 fields
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: String,
    pub policy_hash: String,
    pub step_index: u64,
    pub step_type: Option<String>,
    pub signatures: Option<Vec<crate::types::SignatureWire>>,
    pub state_hash_prev: String,
    pub state_hash_next: String,
    pub chain_digest_prev: String,
    pub chain_digest_next: String,
    pub metrics: crate::types::MetricsWire,
    // V3 Transition Contract fields
    pub objective_result: Option<ObjectiveResult>,
    pub sequence_accumulator: Option<Hash32>, // Replaces boolean sequence_valid (Patch 6)
    pub override_applied: bool,
}

impl Default for MicroReceiptV3Wire {
    fn default() -> Self {
        Self {
            schema_id: "coh.receipt.micro.v3".to_string(),
            version: "1.0.0".to_string(),
            object_id: String::new(),
            canon_profile_hash: String::new(),
            policy_hash: String::new(),
            step_index: 0,
            step_type: None,
            signatures: None,
            state_hash_prev: String::new(),
            state_hash_next: String::new(),
            chain_digest_prev: String::new(),
            chain_digest_next: String::new(),
            metrics: crate::types::MetricsWire::default(),
            objective_result: None,
            sequence_accumulator: None, // New: computed from chain
            override_applied: false,
        }
    }
}

/// Internal V3 receipt type
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MicroReceiptV3 {
    // Base fields
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub step_index: u64,
    pub step_type: Option<String>,
    pub signatures: Option<Vec<crate::types::SignatureWire>>,
    pub state_hash_prev: Hash32,
    pub state_hash_next: Hash32,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    pub metrics: Metrics,
    // V3 Transition Contract fields
    pub objective_result: Option<ObjectiveResult>,
    pub sequence_accumulator: Option<Hash32>, // Replaces boolean sequence_valid (Patch 6)
    pub override_applied: bool,
}

impl Default for MicroReceiptV3 {
    fn default() -> Self {
        Self {
            schema_id: "coh.receipt.micro.v3".to_string(),
            version: "1.0.0".to_string(),
            object_id: String::new(),
            canon_profile_hash: Hash32::default(),
            policy_hash: Hash32::default(),
            step_index: 0,
            step_type: None,
            signatures: None,
            state_hash_prev: Hash32::default(),
            state_hash_next: Hash32::default(),
            chain_digest_prev: Hash32::default(),
            chain_digest_next: Hash32::default(),
            metrics: Metrics::default(),
            objective_result: None,
            sequence_accumulator: None, // New: computed from chain
            override_applied: false,
        }
    }
}

impl TryFrom<MicroReceiptV3Wire> for MicroReceiptV3 {
    type Error = RejectCode;

    fn try_from(w: MicroReceiptV3Wire) -> Result<Self, Self::Error> {
        Ok(MicroReceiptV3 {
            schema_id: w.schema_id,
            version: w.version,
            object_id: w.object_id,
            canon_profile_hash: Hash32::from_hex(&w.canon_profile_hash)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            policy_hash: Hash32::from_hex(&w.policy_hash)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            step_index: w.step_index,
            step_type: w.step_type,
            signatures: w.signatures,
            state_hash_prev: Hash32::from_hex(&w.state_hash_prev)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            state_hash_next: Hash32::from_hex(&w.state_hash_next)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            chain_digest_prev: Hash32::from_hex(&w.chain_digest_prev)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            chain_digest_next: Hash32::from_hex(&w.chain_digest_next)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            metrics: w.metrics.try_into()?,
            // V3 fields
            objective_result: w.objective_result,
            sequence_accumulator: w.sequence_accumulator.clone(),
            override_applied: w.override_applied,
        })
    }
}

impl MicroReceiptV3 {
    /// Check if objective layer is satisfied (null = not checked = pass)
    pub fn objective_satisfied(&self) -> bool {
        match &self.objective_result {
            None => true, // not checked
            Some(result) => match result {
                ObjectiveResult::Satisfied(_) => true,
                ObjectiveResult::Violated(_) => false,
                ObjectiveResult::NotApplicable => true,
            },
        }
    }

    /// Full V3 validity check
    pub fn is_valid(&self) -> bool {
        // V1/V2 checks...
        !self.object_id.is_empty()
            && self.schema_id == "coh.receipt.micro.v3"
            && self.sequence_accumulator.is_some()
            && !self.override_applied
            && self.objective_satisfied()
    }
}

/// Sequence guard configuration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceGuard {
    pub max_cumulative_spend: u128,
    pub window_size: u64,
    pub max_state_drift: u128,
    pub require_monotonicity: bool,
}

impl Default for SequenceGuard {
    fn default() -> Self {
        Self {
            max_cumulative_spend: u128::MAX,
            window_size: 100,
            max_state_drift: u128::MAX,
            require_monotonicity: false,
        }
    }
}

/// Strict sequence guard
pub fn strict_sequence_guard() -> SequenceGuard {
    SequenceGuard {
        max_cumulative_spend: 10_000,
        window_size: 10,
        max_state_drift: 5_000,
        require_monotonicity: true,
    }
}

/// Policy governance configuration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyGovernance {
    pub policy_version: u64,
    pub policy_chain_valid: bool,
    pub allow_overrides: bool,
}

impl Default for PolicyGovernance {
    fn default() -> Self {
        Self {
            policy_version: 0,
            policy_chain_valid: true,
            allow_overrides: false,
        }
    }
}

/// Verification mode
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMode {
    #[default]
    Strict, // Full verification
    Fast,  // Cached/partial
    Async, // Post-check
}

/// Tiered verification config
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TieredConfig {
    pub mode: VerificationMode,
    pub cache_ttl_seconds: u64,
    pub async_queue_size: u64,
}

impl Default for TieredConfig {
    fn default() -> Self {
        Self {
            mode: VerificationMode::Strict,
            cache_ttl_seconds: 0,
            async_queue_size: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v3_default() {
        let wire = MicroReceiptV3Wire::default();
        assert_eq!(wire.schema_id, "coh.receipt.micro.v3");
        assert!(wire.sequence_accumulator.is_some()); // V3 uses sequence_accumulator (Patch 6)
        assert!(!wire.override_applied);
    }

    #[test]
    fn test_objective_satisfied() {
        let mut receipt = MicroReceiptV3::default();
        assert!(receipt.objective_satisfied());

        receipt.objective_result = Some(ObjectiveResult::Violated(ObjectiveTarget::MinimizeSpend));
        assert!(!receipt.objective_satisfied());
    }

    #[test]
    fn test_sequence_guard_defaults() {
        let guard = SequenceGuard::default();
        assert_eq!(guard.window_size, 100);
    }
}

// =============================================================================
// V3 Canonical Mapping (Patch 5)
// - All accept/reject-relevant V3 fields must be digest-bound
// =============================================================================

use sha2::{Digest, Sha256};

/// V3 canonical prehash - all fields that affect accept/reject decisions
///
/// Per Patch 5: V3 fields that affect accept/reject MUST be digest-bound.
/// If objective_result, sequence_valid, or override_applied can be changed without
/// changing the digest, V3 is still projected (attacker-writable).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MicroReceiptV3Prehash {
    // Base identification
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub step_index: u64,

    // Policy binding (already affects accept/reject)
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,

    // Accounting (already in digest)
    pub v_pre: u128,
    pub v_post: u128,
    pub spend: u128,
    pub defect: u128,
    pub authority: u128,
    pub d_slack: u128, // Delta slack from metrics

    // State binding (already in digest)
    pub state_hash_prev: Hash32,
    pub state_hash_next: Hash32,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,

    // V3 Transition Contract fields (NEW - must be digest-bound per Patch 5)
    pub objective_result: Option<ObjectiveResult>,
    pub sequence_accumulator: Option<Hash32>, // Replaces boolean sequence_valid
    pub override_applied: bool,
}

impl MicroReceiptV3Prehash {
    /// Create from V3 receipt (all fields)
    pub fn from_receipt(r: &MicroReceiptV3) -> Self {
        Self {
            schema_id: r.schema_id.clone(),
            version: r.version.clone(),
            object_id: r.object_id.clone(),
            step_index: r.step_index,
            canon_profile_hash: r.canon_profile_hash,
            policy_hash: r.policy_hash,
            v_pre: r.metrics.v_pre,
            v_post: r.metrics.v_post,
            spend: r.metrics.spend,
            defect: r.metrics.defect,
            authority: r.metrics.authority,
            d_slack: r.metrics.d_slack,
            state_hash_prev: r.state_hash_prev,
            state_hash_next: r.state_hash_next,
            chain_digest_prev: r.chain_digest_prev,
            chain_digest_next: r.chain_digest_next,
            // V3 Transition Contract fields
            objective_result: r.objective_result.clone(),
            // Note: sequence_accumulator must be computed externally
            sequence_accumulator: None,
            override_applied: r.override_applied,
        }
    }

    /// Compute V3 canonical CORE digest - excludes sequence_accumulator
    /// This is used to compute the sequence accumulator (non-circular).
    /// Returns error if serialization fails - verifier must reject, not silently continue.
    pub fn canonical_core_digest(&self) -> Result<Hash32, RejectCode> {
        let mut hasher = Sha256::new();
        hasher.update(b"COH_V3_CORE");

        // Base identification
        hasher.update(self.schema_id.as_bytes());
        hasher.update(self.version.as_bytes());
        hasher.update(self.object_id.as_bytes());
        hasher.update(self.step_index.to_be_bytes());

        // Policy binding
        hasher.update(self.canon_profile_hash.0);
        hasher.update(self.policy_hash.0);

        // Accounting
        hasher.update(self.v_pre.to_be_bytes());
        hasher.update(self.v_post.to_be_bytes());
        hasher.update(self.spend.to_be_bytes());
        hasher.update(self.defect.to_be_bytes());
        hasher.update(self.authority.to_be_bytes());
        hasher.update(self.d_slack.to_be_bytes());

        // State binding
        hasher.update(self.state_hash_prev.0);
        hasher.update(self.state_hash_next.0);
        hasher.update(self.chain_digest_prev.0);
        hasher.update(self.chain_digest_next.0);

        // V3 Transition Contract fields - EXCLUDE sequence_accumulator for core digest
        if let Some(ref obj) = self.objective_result {
            hasher.update(b"objective_result");
            let obj_bytes = serde_json::to_vec(obj).map_err(|_| RejectCode::RejectSchema)?;
            hasher.update((obj_bytes.len() as u64).to_be_bytes());
            hasher.update(obj_bytes);
        }
        hasher.update(b"override_applied");
        hasher.update([self.override_applied as u8]);

        Ok(Hash32(hasher.finalize().into()))
    }

    /// Compute V3 canonical FULL digest - includes sequence_accumulator
    /// Use this only AFTER sequence verification, for display/chain binding.
    pub fn canonical_full_digest(&self) -> Result<Hash32, RejectCode> {
        // Start with core digest
        let core = self.canonical_core_digest()?;

        let mut hasher = Sha256::new();
        hasher.update(b"COH_V3_FULL");
        hasher.update(core.0);

        // Now include sequence accumulator (already verified)
        if let Some(ref seq) = self.sequence_accumulator {
            hasher.update(b"sequence_accumulator");
            hasher.update(seq.0);
        }

        Ok(Hash32(hasher.finalize().into()))
    }
}

/// Compute V3 core canonical digest from wire receipt
/// Returns Err if parsing fails or canonicalization fails.
/// Uses CORE digest (excludes sequence_accumulator) to avoid circular dependency.
pub fn compute_v3_canonical_digest(wire: &MicroReceiptV3Wire) -> Result<Hash32, RejectCode> {
    // Parse to internal type first
    let internal: MicroReceiptV3 = wire
        .clone()
        .try_into()
        .map_err(|_| RejectCode::RejectSchema)?;
    let prehash = MicroReceiptV3Prehash::from_receipt(&internal);

    // Use CORE digest - excludes sequence to break circular dependency
    prehash.canonical_core_digest()
}

#[cfg(test)]
mod v3_canonical_tests {
    use super::*;

    #[test]
    fn test_v3_prehash_digest_binding_objective() {
        // Given: base prehash
        let prehash = MicroReceiptV3Prehash {
            schema_id: "coh.receipt.micro.v3".to_string(),
            version: "1.0.0".to_string(),
            object_id: "test".to_string(),
            step_index: 0,
            canon_profile_hash: Hash32::default(),
            policy_hash: Hash32::default(),
            v_pre: 1000,
            v_post: 900,
            spend: 100,
            defect: 0,
            authority: 200,
            d_slack: 0,
            state_hash_prev: Hash32::default(),
            state_hash_next: Hash32::default(),
            chain_digest_prev: Hash32::default(),
            chain_digest_next: Hash32::default(),
            objective_result: None,
            sequence_accumulator: None,
            override_applied: false,
        };

        let digest1 = prehash.canonical_core_digest();

        // When: objective_result changes
        let mut modified = prehash.clone();
        modified.objective_result = Some(ObjectiveResult::Violated(ObjectiveTarget::MinimizeSpend));

        let digest2 = modified.canonical_core_digest();

        // Then: digest MUST change (tamper test)
        assert_ne!(digest1, digest2, "objective_result MUST affect digest");
    }

    #[test]
    fn test_v3_prehash_digest_binding_override() {
        // Given: base prehash
        let prehash = MicroReceiptV3Prehash {
            schema_id: "coh.receipt.micro.v3".to_string(),
            version: "1.0.0".to_string(),
            object_id: "test".to_string(),
            step_index: 0,
            canon_profile_hash: Hash32::default(),
            policy_hash: Hash32::default(),
            v_pre: 1000,
            v_post: 900,
            spend: 100,
            defect: 0,
            authority: 200,
            d_slack: 0,
            state_hash_prev: Hash32::default(),
            state_hash_next: Hash32::default(),
            chain_digest_prev: Hash32::default(),
            chain_digest_next: Hash32::default(),
            objective_result: None,
            sequence_accumulator: None,
            override_applied: false,
        };

        let digest1 = prehash.canonical_core_digest();

        // When: override_applied changes (attacker tries to bypass)
        let mut modified = prehash.clone();
        modified.override_applied = true;

        let digest2 = modified.canonical_core_digest();

        // Then: digest MUST change (tamper test)
        assert_ne!(digest1, digest2, "override_applied MUST affect digest");
    }

    #[test]
    fn test_v3_prehash_sequence_accumulator_binding() {
        // Given: base prehash without sequence
        let prehash = MicroReceiptV3Prehash {
            schema_id: "coh.receipt.micro.v3".to_string(),
            version: "1.0.0".to_string(),
            object_id: "test".to_string(),
            step_index: 42,
            canon_profile_hash: Hash32::default(),
            policy_hash: Hash32::default(),
            v_pre: 1000,
            v_post: 900,
            spend: 100,
            defect: 0,
            authority: 200,
            d_slack: 0,
            state_hash_prev: Hash32::default(),
            state_hash_next: Hash32::default(),
            chain_digest_prev: Hash32::default(),
            chain_digest_next: Hash32::default(),
            objective_result: None,
            sequence_accumulator: None,
            override_applied: false,
        };

        let digest1 = prehash.canonical_core_digest();

        // When: sequence_accumulator added (replaces boolean sequence_valid)
        let mut modified = prehash.clone();
        modified.sequence_accumulator = Some(Hash32([1; 32]));

        let digest2 = modified.canonical_core_digest();

        // Then: digest MUST change
        assert_ne!(digest1, digest2, "sequence_accumulator MUST affect digest");
    }
}
