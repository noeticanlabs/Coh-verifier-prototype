//! V3 Canonical and Projection Bindings
//!
//! Per audit Patch 5:
//! - V3 is canonical: all fields affect digest
//! - L0 projection exists only for legacy compatibility
//!
//! Field binding classification:
//! | Field | Binding? | Why |
//! |-------|----------|-----|
//! | version | YES | prevents downgrade |
//! | profile | YES | selects law set |
//! | spend | YES | accounting |
//! | defect_reserve | YES | accounting |
//! | authority | YES | accounting |
//! | raw_defect | YES if envelope enforced | evidence |
//! | delta_hat | YES | envelope |
//! | defect_cap | YES | policy cap |
//! | objective | YES if affects admissibility | governance |
//! | sequence_accumulator | YES | chain order |
//! | display labels | NO | UI only |

use crate::types::{Hash32, MicroReceipt};

/// V3 field binding classification
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldBinding {
    /// Field is digest-bound and affects admissibility
    Binding,
    /// Field affects admission decision
    DecisionRelevant,
    /// Non-binding metadata only
    NonBinding,
}

/// V3 field classification table
pub fn classify_v3_field(field: &str) -> FieldBinding {
    match field {
        // Binding fields (in canonical digest)
        "version"
        | "profile"
        | "spend"
        | "defect_reserve"
        | "authority"
        | "delta_hat"
        | "defect_cap"
        | "sequence_accumulator"
        | "objective"
        | "state_pre"
        | "state_post"
        | "chain_digest_prev"
        | "chain_digest_next"
        | "object_id"
        | "policy_hash" => FieldBinding::Binding,

        // Decision-relevant (not in digest but affects accept)
        "raw_defect" => FieldBinding::DecisionRelevant,

        // UI-only, no cryptographic binding
        "display_name" | "description" | "display_metadata" => FieldBinding::NonBinding,

        // Default to binding for safety (err on side of caution)
        _ => FieldBinding::Binding,
    }
}

/// MicroReceiptV3 canonical representation
#[derive(Clone, Debug)]
pub struct MicroReceiptV3 {
    /// Binding fields
    pub version: u16,
    pub profile: String,
    pub object_id: String,

    // Accounting
    pub spend: u128,
    pub defect_reserve: u128,
    pub authority: u128,

    // Semantic envelope
    pub raw_defect: Option<u128>,
    pub delta_hat: Option<u128>,
    pub defect_cap: Option<u128>,

    // Governance
    pub objective: Option<String>,

    // Chain order
    pub sequence_accumulator: Option<Hash32>,

    // State binding
    pub state_pre: Hash32,
    pub state_post: Hash32,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,

    // Policy
    pub policy_hash: Hash32,
}

impl MicroReceiptV3 {
    /// Construct from generic MicroReceipt
    pub fn from_receipt(r: &MicroReceipt) -> Self {
        Self {
            version: r.version,
            profile: r.profile.clone(),
            object_id: r.object_id.clone(),
            spend: r.metrics.spend,
            defect_reserve: r.metrics.defect,
            authority: r.metrics.authority,
            raw_defect: None, // Not in base receipt
            delta_hat: Some(r.metrics.delta_hat),
            defect_cap: None,
            objective: None,
            sequence_accumulator: None,
            state_pre: r.metrics.state_pre,
            state_post: r.metrics.state_post,
            chain_digest_prev: r.metrics.chain_digest_prev,
            chain_digest_next: r.chain_digest_next,
            policy_hash: r.policy_hash,
        }
    }

    /// Compute canonical prehash for V3 digest
    /// All binding fields included
    pub fn canonical_prehash(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(b"COH_V3_CANON");

        // Version/profile
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.profile.as_bytes());

        // Accounting
        hasher.update(self.spend.to_be_bytes());
        hasher.update(self.defect_reserve.to_be_bytes());
        hasher.update(self.authority.to_be_bytes());

        // Semantic envelope (if present)
        if let Some(dh) = self.delta_hat {
            hasher.update(dh.to_be_bytes());
        }
        if let Some(dc) = self.defect_cap {
            hasher.update(dc.to_be_bytes());
        }

        // Governance (if affects admissibility)
        if let Some(ref obj) = self.objective {
            hasher.update(obj.as_bytes());
        }

        // Sequence (if present)
        if let Some(seq) = self.sequence_accumulator {
            hasher.update(seq.0);
        }

        // State binding
        hasher.update(self.state_pre.0);
        hasher.update(self.state_post.0);

        // Chain continuity
        hasher.update(self.chain_digest_prev.0);
        hasher.update(self.chain_digest_next.0);

        hasher.update(self.policy_hash.0);

        hasher.finalize().to_vec()
    }

    /// Project to L0 for compatibility only
    /// Returns base MicroReceipt with mapped fields
    pub fn project_to_l0(&self) -> MicroReceipt {
        MicroReceipt {
            version: self.version,
            profile: self.profile.clone(),
            schema_id: "V3_L0_PROJECTION".to_string(),
            object_id: self.object_id.clone(),
            step_type: Some("coh.step.projection".to_string()),
            metrics: crate::types::MetricsPrehash {
                v_pre: 0, // Not in V3 representation
                v_post: 0,
                spend: self.spend,
                defect: self.defect_reserve, // map reserve to defect
                delta_hat: self.delta_hat.unwrap_or(0),
                authority: self.authority,
                state_pre: self.state_pre,
                state_post: self.state_post,
                chain_digest_prev: self.chain_digest_prev,
                chain_digest_next: self.chain_digest_next,
                d_slack: 0,
                projection_hash: Hash32([0; 32]),
                pl_tau: 0,
            },
            policy_hash: self.policy_hash,
            step_index: 0,
            chain_digest_next: self.chain_digest_next,
            sequence_valid: true, // Can't verify sequence in L0 projection
            ..Default::default()
        }
    }

    /// Check if field is binding (affects digest or admission)
    pub fn is_binding_field(&self, field_name: &str) -> bool {
        match classify_v3_field(field_name) {
            FieldBinding::Binding | FieldBinding::DecisionRelevant => true,
            FieldBinding::NonBinding => false,
        }
    }
}
