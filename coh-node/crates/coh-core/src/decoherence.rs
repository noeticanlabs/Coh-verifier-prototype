use crate::atom::CohAtom;
use crate::types::{Hash32, DomainId, Signature};
use serde::{Deserialize, Serialize};
use num_rational::Rational64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecoherenceState {
    Coherent,       // entanglement intact
    Weakening,      // coupling degrading but still valid
    SplitCertified, // safely separated into local-valid atoms
    Quarantined,    // cannot split safely
    Rejected,       // invalid coupling or unsafe mutation attempt
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecoherenceCause {
    CouplingWitnessExpired,
    MonogamyScopeConflict,
    PolicyChanged,
    DomainChanged,
    SharedDefectExhausted,
    SharedAuthorityExhausted,
    ParticipantInvalidated,
    PhaseDivergence,
    AnchorConeExit,
    ManualSeverance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecoherenceMode {
    HardSplit,
    AssistedSplit,
    QuarantineOnly,
    AuditOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoherenceContext {
    pub mode: DecoherenceMode,
    pub policy_hash: Hash32,
    pub domain_id: DomainId,
    pub allow_assisted_split: bool,
    pub production: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityGrant {
    pub atom_id: Hash32,
    pub authority: Rational64,
    pub receipt_hash: Hash32,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoherenceCertificate {
    pub version: u16,

    pub entanglement_hash: Hash32,
    pub cause: DecoherenceCause,

    pub pre_joint_margin: Rational64,
    pub post_local_margins: Vec<Rational64>,

    pub released_shared_defect: Rational64,
    pub released_shared_authority: Rational64,

    pub participant_atom_hashes: Vec<Hash32>,

    pub split_witness_hash: Hash32,
    pub policy_hash: Hash32,
    pub domain_id: DomainId,

    pub decoherence_hash: Hash32,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineReceipt {
    pub entanglement_hash: Hash32,
    pub failed_participants: Vec<Hash32>,
    pub failed_margins: Vec<Rational64>,
    pub cause: DecoherenceCause,
    pub policy_hash: Hash32,
    pub domain_id: DomainId,
    pub quarantine_hash: Hash32,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoherenceResult {
    pub state: DecoherenceState,
    pub certificate: Option<DecoherenceCertificate>,
    pub local_atoms: Vec<CohAtom>,
    pub quarantine_receipt: Option<QuarantineReceipt>,
}

#[derive(Debug, thiserror::Error)]
pub enum DecoherenceReject {
    #[error("Invalid entanglement")]
    InvalidEntanglement,
    #[error("Missing cause")]
    MissingCause,
    #[error("Domain mismatch")]
    DomainMismatch,
    #[error("Policy mismatch")]
    PolicyMismatch,
    #[error("Shared defect redistribution attempt")]
    SharedDefectRedistributionAttempt,
    #[error("Shared authority redistribution attempt")]
    SharedAuthorityRedistributionAttempt,
    #[error("Local margin negative")]
    LocalMarginNegative,
    #[error("Assisted authority missing")]
    AssistedAuthorityMissing,
    #[error("Assisted authority invalid")]
    AssistedAuthorityInvalid,
    #[error("Split witness invalid")]
    SplitWitnessInvalid,
    #[error("Monogamy scope still active")]
    MonogamyScopeStillActive,
    #[error("Unauthorized verifier")]
    UnauthorizedVerifier,
    #[error("Bad decoherence hash")]
    BadDecoherenceHash,
    #[error("Non-canonical encoding")]
    NonCanonicalEncoding,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntangledThreadState {
    Coherent,
    Decohered,
    Quarantined,
}
