use crate::atom::CohAtom;
use crate::types::{Hash32, DomainId, Signature};
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use sha2::Digest;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecoherenceState {
    Coherent,       // entanglement intact
    Weakening,      // coupling degrading but still valid
    SplitCertified, // safely separated into local-valid atoms
    Quarantined,    // cannot split safely
    Burned,         // budget nullified
    Rejected,       // invalid coupling or unsafe mutation attempt
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MonogamyState {
    Active,
    Decohered,
    Burned,
    Quarantined,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntanglementMode {
    Fixture,
    Heuristic,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntanglementContext {
    pub mode: EntanglementMode,
    pub domain_id: DomainId,
    pub policy_hash: Hash32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoherenceContext {
    pub mode: DecoherenceMode,
    pub policy_hash: Hash32,
    pub domain_id: DomainId,
    pub allow_assisted_split: bool,
    pub entanglement_mode: EntanglementMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityGrant {
    pub atom_id: Hash32,
    pub authority: Rational64,
    pub receipt_hash: Hash32,
    pub signer: Hash32,
    pub domain_id: DomainId,
    pub policy_hash: Hash32,
    pub expires_at: u64,
    pub grant_hash: Hash32,
    pub signature: Signature,
}

impl AuthorityGrant {
    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"authoritygrant:v1");
        hasher.update(self.atom_id.0);
        hasher.update(self.authority.reduced().numer().to_be_bytes());
        hasher.update(self.authority.reduced().denom().to_be_bytes());
        hasher.update(self.receipt_hash.0);
        hasher.update(self.signer.0);
        hasher.update(self.domain_id.0.0);
        hasher.update(self.policy_hash.0);
        hasher.update(self.expires_at.to_be_bytes());
        Hash32(hasher.finalize().into())
    }

    pub fn hash_valid(&self) -> bool {
        self.grant_hash == self.canonical_hash()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoherenceCertificate {
    pub version: u16,

    pub entanglement_hash: Hash32,
    pub cause: DecoherenceCause,

    pub pre_joint_margin: Rational64,
    pub post_local_margins: Vec<Rational64>,

    pub burned_shared_defect: Rational64,
    pub burned_shared_authority: Rational64,
    pub redistributed_shared_defect: Rational64,
    pub redistributed_shared_authority: Rational64,

    pub participant_atom_hashes: Vec<Hash32>,

    pub split_witness_hash: Hash32,
    pub policy_hash: Hash32,
    pub domain_id: DomainId,

    pub decoherence_hash: Hash32,
    pub signature: Signature,
}

impl DecoherenceCertificate {
    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"decoherencecert:v1");
        hasher.update(self.entanglement_hash.0);
        hasher.update([self.cause as u8]);
        
        let mut update_rat = |r: &Rational64| {
            hasher.update(r.reduced().numer().to_be_bytes());
            hasher.update(r.reduced().denom().to_be_bytes());
        };

        update_rat(&self.pre_joint_margin);
        for m in &self.post_local_margins { update_rat(m); }
        
        update_rat(&self.burned_shared_defect);
        update_rat(&self.burned_shared_authority);
        update_rat(&self.redistributed_shared_defect);
        update_rat(&self.redistributed_shared_authority);

        for h in &self.participant_atom_hashes { hasher.update(h.0); }
        
        hasher.update(self.split_witness_hash.0);
        hasher.update(self.policy_hash.0);
        hasher.update(self.domain_id.0.0);
        
        Hash32(hasher.finalize().into())
    }
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

impl QuarantineReceipt {
    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"quarantinereceipt:v1");
        hasher.update(self.entanglement_hash.0);
        for p in &self.failed_participants { hasher.update(p.0); }
        for m in &self.failed_margins {
            hasher.update(m.reduced().numer().to_be_bytes());
            hasher.update(m.reduced().denom().to_be_bytes());
        }
        hasher.update([self.cause as u8]);
        hasher.update(self.policy_hash.0);
        hasher.update(self.domain_id.0.0);
        Hash32(hasher.finalize().into())
    }
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
