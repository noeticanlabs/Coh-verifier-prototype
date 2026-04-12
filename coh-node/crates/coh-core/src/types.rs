pub use crate::reject::RejectCode;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    pub fn from_hex(hex: &str) -> Result<Self, RejectCode> {
        // Overbuilt: Explicitly check length before decoding to provide cleaner RejectNumericParse
        if hex.len() != 64 {
            return Err(RejectCode::RejectNumericParse);
        }
        let bytes = hex::decode(hex).map_err(|_| RejectCode::RejectNumericParse)?;
        if bytes.len() != 32 {
            // Should be covered by hex length 64, but safety first
            return Err(RejectCode::RejectNumericParse);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash32(arr))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Decision {
    Accept,
    Reject,
    SlabBuilt,
}

// --- Wire Layer ---

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MetricsWire {
    pub v_pre: String,
    pub v_post: String,
    pub spend: String,
    pub defect: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MicroReceiptWire {
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: String,
    pub policy_hash: String,
    pub step_index: u64,
    /// Optional step type for categorization (e.g., "generate", "review", "execute")
    #[serde(default)]
    pub step_type: Option<String>,
    /// Optional signatures for multi-party approval
    #[serde(default)]
    pub signatures: Option<Vec<SignatureWire>>,
    pub state_hash_prev: String,
    pub state_hash_next: String,
    pub chain_digest_prev: String,
    pub chain_digest_next: String,
    pub metrics: MetricsWire,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlabSummaryWire {
    pub total_spend: String,
    pub total_defect: String,
    pub v_pre_first: String,
    pub v_post_last: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlabReceiptWire {
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: String,
    pub policy_hash: String,
    pub range_start: u64,
    pub range_end: u64,
    pub micro_count: u64,
    pub chain_digest_prev: String,
    pub chain_digest_next: String,
    pub state_hash_first: String,
    pub state_hash_last: String,
    pub merkle_root: String,
    pub summary: SlabSummaryWire,
}

// --- Runtime Layer ---

pub struct Metrics {
    pub v_pre: u128,
    pub v_post: u128,
    pub spend: u128,
    pub defect: u128,
}

pub struct MicroReceipt {
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub step_index: u64,
    /// Optional step type for categorization (e.g., "generate", "review", "execute")
    pub step_type: String,
    /// Optional signatures for multi-party approval
    pub signatures: Option<Vec<Signature>>,
    pub state_hash_prev: Hash32,
    pub state_hash_next: Hash32,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    pub metrics: Metrics,
}

pub struct SlabSummary {
    pub total_spend: u128,
    pub total_defect: u128,
    pub v_pre_first: u128,
    pub v_post_last: u128,
}

pub struct SlabReceipt {
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub range_start: u64,
    pub range_end: u64,
    pub micro_count: u64,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    pub state_hash_first: Hash32,
    pub state_hash_last: Hash32,
    pub merkle_root: Hash32,
    pub summary: SlabSummary,
}

// --- Prehash Layer (Alphabetized) ---

#[derive(Serialize)]
pub struct MetricsPrehash {
    pub defect: String,
    pub spend: String,
    pub v_post: String,
    pub v_pre: String,
}

#[derive(Serialize)]
pub struct MicroReceiptPrehash {
    pub canon_profile_hash: String,
    pub chain_digest_prev: String,
    pub metrics: MetricsPrehash,
    pub object_id: String,
    pub policy_hash: String,
    pub schema_id: String,
    pub signatures: Option<Vec<SignaturePrehash>>,
    pub state_hash_next: String,
    pub state_hash_prev: String,
    pub step_index: u64,
    pub step_type: String,
    pub version: String,
}

/// Signature prehash for canonicalization
#[derive(Serialize)]
pub struct SignaturePrehash {
    pub signer: String,
    pub signature: String,
    pub timestamp: String,
}

// --- Result Layer ---

#[derive(Serialize)]
pub struct VerifyMicroResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_digest_next: Option<String>,
}

#[derive(Serialize)]
pub struct VerifyChainResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    pub steps_verified: u64,
    pub first_step_index: u64,
    pub last_step_index: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_chain_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing_step_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps_verified_before_failure: Option<u64>,
}

#[derive(Serialize)]
pub struct BuildSlabResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_end: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub micro_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merkle_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slab: Option<SlabReceiptWire>,
}

#[derive(Serialize)]
pub struct VerifySlabResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    pub range_start: u64,
    pub range_end: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub micro_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merkle_root: Option<String>,
}

// --- Conversions ---

fn parse_u128(s: &str) -> Result<u128, RejectCode> {
    s.parse::<u128>()
        .map_err(|_| RejectCode::RejectNumericParse)
}

impl TryFrom<MetricsWire> for Metrics {
    type Error = RejectCode;
    fn try_from(w: MetricsWire) -> Result<Self, Self::Error> {
        Ok(Metrics {
            v_pre: parse_u128(&w.v_pre)?,
            v_post: parse_u128(&w.v_post)?,
            spend: parse_u128(&w.spend)?,
            defect: parse_u128(&w.defect)?,
        })
    }
}

/// Signature wire for multi-party signing support
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignatureWire {
    pub signer: String,
    pub signature: String,
    pub timestamp: u64,
}

/// Helper macro to create MicroReceiptWire with required fields only
/// Usage: micro_wire!(schema_id, version, object_id, ...)
#[macro_export]
macro_rules! micro_wire {
    ($s:expr, $v:expr, $o:expr, $cph:expr, $polh:expr, $si:expr, $shp:expr, $shn:expr, $cdp:expr, $cdn:expr, $m:expr) => {
        MicroReceiptWire {
            schema_id: $s,
            version: $v,
            object_id: $o,
            canon_profile_hash: $cph,
            policy_hash: $polh,
            step_index: $si,
            step_type: None,
            signatures: None,
            state_hash_prev: $shp,
            state_hash_next: $shn,
            chain_digest_prev: $cdp,
            chain_digest_next: $cdn,
            metrics: $m,
        }
    };
}

/// Signature runtime type
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub signer: String,
    pub signature: Hash32,
    pub timestamp: u64,
}

impl TryFrom<SignatureWire> for Signature {
    type Error = RejectCode;
    fn try_from(w: SignatureWire) -> Result<Self, Self::Error> {
        Ok(Signature {
            signer: w.signer,
            signature: Hash32::from_hex(&w.signature)?,
            timestamp: w.timestamp,
        })
    }
}

impl TryFrom<MicroReceiptWire> for MicroReceipt {
    type Error = RejectCode;
    fn try_from(w: MicroReceiptWire) -> Result<Self, Self::Error> {
        // Parse optional signatures
        let signatures = match w.signatures {
            Some(sigs) => {
                let mut parsed = Vec::with_capacity(sigs.len());
                for s in sigs {
                    parsed.push(Signature::try_from(s)?);
                }
                Some(parsed)
            }
            None => None,
        };

        Ok(MicroReceipt {
            schema_id: w.schema_id,
            version: w.version,
            object_id: w.object_id,
            canon_profile_hash: Hash32::from_hex(&w.canon_profile_hash)?,
            policy_hash: Hash32::from_hex(&w.policy_hash)?,
            step_index: w.step_index,
            // Optional fields - uses unwrap_or_default for backward compatibility
            step_type: w.step_type.unwrap_or_default(),
            signatures,
            state_hash_prev: Hash32::from_hex(&w.state_hash_prev)?,
            state_hash_next: Hash32::from_hex(&w.state_hash_next)?,
            chain_digest_prev: Hash32::from_hex(&w.chain_digest_prev)?,
            chain_digest_next: Hash32::from_hex(&w.chain_digest_next)?,
            metrics: Metrics::try_from(w.metrics)?,
        })
    }
}

impl TryFrom<SlabSummaryWire> for SlabSummary {
    type Error = RejectCode;
    fn try_from(w: SlabSummaryWire) -> Result<Self, Self::Error> {
        Ok(SlabSummary {
            total_spend: parse_u128(&w.total_spend)?,
            total_defect: parse_u128(&w.total_defect)?,
            v_pre_first: parse_u128(&w.v_pre_first)?,
            v_post_last: parse_u128(&w.v_post_last)?,
        })
    }
}

impl TryFrom<SlabReceiptWire> for SlabReceipt {
    type Error = RejectCode;
    fn try_from(w: SlabReceiptWire) -> Result<Self, Self::Error> {
        Ok(SlabReceipt {
            schema_id: w.schema_id,
            version: w.version,
            object_id: w.object_id,
            canon_profile_hash: Hash32::from_hex(&w.canon_profile_hash)?,
            policy_hash: Hash32::from_hex(&w.policy_hash)?,
            range_start: w.range_start,
            range_end: w.range_end,
            micro_count: w.micro_count,
            chain_digest_prev: Hash32::from_hex(&w.chain_digest_prev)?,
            chain_digest_next: Hash32::from_hex(&w.chain_digest_next)?,
            state_hash_first: Hash32::from_hex(&w.state_hash_first)?,
            state_hash_last: Hash32::from_hex(&w.state_hash_last)?,
            merkle_root: Hash32::from_hex(&w.merkle_root)?,
            summary: SlabSummary::try_from(w.summary)?,
        })
    }
}
