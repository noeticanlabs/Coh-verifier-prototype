use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectCode {
    RejectSchema,
    RejectCanonProfile,
    RejectChainDigest,
    RejectStateHashLink,
    RejectNumericParse,
    RejectOverflow,
    RejectPolicyViolation,
    RejectSlabSummary,
    RejectSlabMerkle,
}
