use crate::reject::RejectCode;
use crate::types::{MetricsPrehash, MicroReceipt, MicroReceiptPrehash};

pub struct SchemaRegistry;

impl SchemaRegistry {
    pub const MICRO_V1_ID: &'static str = "coh.receipt.micro.v1";
    pub const MICRO_V1_VERSION: &'static str = "1.0.0";
    pub const SLAB_V1_ID: &'static str = "coh.receipt.slab.v1";
    pub const SLAB_V1_VERSION: &'static str = "1.0.0";
    pub const CANON_PROFILE_V1: &'static str =
        "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

    pub fn validate_micro(schema_id: &str, version: &str) -> bool {
        schema_id == Self::MICRO_V1_ID && version == Self::MICRO_V1_VERSION
    }

    pub fn validate_slab(schema_id: &str, version: &str) -> bool {
        schema_id == Self::SLAB_V1_ID && version == Self::SLAB_V1_VERSION
    }

    pub fn validate_profile(profile_hash: &str) -> bool {
        profile_hash == Self::CANON_PROFILE_V1
    }
}

pub use SchemaRegistry as CanonRegistry;

pub fn to_prehash_view(r: &MicroReceipt) -> MicroReceiptPrehash {
    MicroReceiptPrehash {
        canon_profile_hash: r.canon_profile_hash.to_hex(),
        chain_digest_prev: r.chain_digest_prev.to_hex(),
        metrics: MetricsPrehash {
            authority: r.metrics.authority.to_string(),
            v_pre: r.metrics.v_pre.to_string(),
            v_post: r.metrics.v_post.to_string(),
            spend: r.metrics.spend.to_string(),
            defect: r.metrics.defect.to_string(),
            m_pre: r.metrics.m_pre.to_string(),
            m_post: r.metrics.m_post.to_string(),
            c_cost: r.metrics.c_cost.to_string(),
            d_slack: r.metrics.d_slack.to_string(),
            projection_hash: r.metrics.projection_hash.to_hex(),
            pl_tau: r.metrics.pl_tau.to_string(),
            pl_budget: r.metrics.pl_budget.to_string(),
            pl_provenance: r.metrics.pl_provenance.to_string(),
        },
        object_id: r.object_id.clone(),
        policy_hash: r.policy_hash.to_hex(),
        profile: match r.profile {
            crate::types::AdmissionProfile::CoherenceOnlyV1 => "COHERENCE_ONLY_V1".to_string(),
            crate::types::AdmissionProfile::FormationV2 => "FORMATION_V2".to_string(),
        },
        schema_id: r.schema_id.clone(),
        state_hash_next: r.state_hash_next.to_hex(),
        state_hash_prev: r.state_hash_prev.to_hex(),
        step_index: r.step_index,
        step_type: r.step_type.clone(),
        version: r.version.clone(),
    }
}

pub fn to_canonical_json_bytes<T: serde::Serialize>(val: &T) -> Result<Vec<u8>, RejectCode> {
    // JCS Compliance Note:
    // Since our Prehash structs ensure alphabetical field order and we use
    // serde_json::to_vec (which omits whitespace), this is JCS-compatible
    // for our current schema (which uses Strings for all potentially
    // ambiguous numeric/special types).
    serde_json::to_vec(val).map_err(|_| RejectCode::RejectNumericParse)
}

/// Strict RFC 8785 canonical JSON serialization
///
/// RFC 8785 requires:
/// 1. Object keys sorted lexicographically
/// 2. No whitespace insignificant to the value
/// 3. Numbers encoded as decimal strings (no exponent)
/// 4. String content escaped per RFC 8785
///
/// Current implementation is JCS-style (close but not strictly RFC 8785).
/// To enable strict RFC 8785, use an external crate like:
/// - `canonical-json` (not currently available on crates.io)
/// - Custom implementation
///
/// Build with `--features strict-canonical` to enable strict mode
/// (requires adding an appropriate crate to Cargo.toml)
#[cfg(feature = "strict-canonical")]
pub fn to_strict_canonical_json_bytes<T: serde::Serialize>(val: &T) -> Result<Vec<u8>, RejectCode> {
    // RFC 8785: Canonical JSON requires deterministic encoding
    // json-canonical handles: string ordering, number encoding, whitespace
    let bytes = serde_json::to_vec(val).map_err(|_| RejectCode::RejectNumericParse)?;
    json_canonical::serialize(&bytes).map_err(|_| RejectCode::RejectNumericParse)
}

#[cfg(not(feature = "strict-canonical"))]
pub fn to_strict_canonical_json_bytes<T: serde::Serialize>(val: &T) -> Result<Vec<u8>, RejectCode> {
    // Fallback: use JCS-style when strict-canonical not enabled
    to_canonical_json_bytes(val)
}
