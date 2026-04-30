use crate::types::{Hash32, DomainId, RvStatus, Signature, Timestamp};
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use num_traits::ToPrimitive;
use sha2::Digest;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CohBitState {
    Superposed,
    ConditionedContinuation,
    Rejected,
    Deferred,
}

#[derive(Debug, Error, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CohBitReject {
    #[error("Non-canonical encoding")]
    NonCanonicalEncoding,
    #[error("Bad receipt hash")]
    BadReceiptHash,
    #[error("Bad signature")]
    BadSignature,
    #[error("Unknown verifier")]
    UnknownVerifier,
    #[error("Policy mismatch")]
    PolicyMismatch,
    #[error("Certificate missing")]
    CertificateMissing,
    #[error("Certificate rejected")]
    CertificateRejected,
    #[error("Defect exceeds delta-hat")]
    DefectExceedsDeltaHat,
    #[error("Negative margin")]
    NegativeMargin,
    #[error("Negative spend")]
    NegativeSpend,
    #[error("Negative authority")]
    NegativeAuthority,
    #[error("Probability out of range")]
    ProbabilityOutOfRange,
    #[error("State kind mismatch")]
    StateKindMismatch,
    #[error("State hash mismatch")]
    StateHashMismatch,
    #[error("Chain index mismatch")]
    ChainIndexMismatch,
    #[error("Chain digest mismatch")]
    ChainDigestMismatch,
    #[error("Previous receipt mismatch")]
    PreviousReceiptMismatch,
    #[error("Authority expired")]
    AuthorityExpired,
    #[error("Authority scope violation")]
    AuthorityScopeViolation,
    #[error("Authority cap exceeded")]
    AuthorityCapExceeded,
    #[error("Execution mismatch")]
    ExecutionMismatch,
    #[error("Parallel conflict")]
    ParallelConflict,
    #[error("Unsupported version")]
    UnsupportedVersion,
}

/// A Rejected Proposal Record (Instability Data)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RejectReceipt {
    pub attempted_from_state: Hash32,
    pub attempted_action_hash: Hash32,
    pub attempted_to_state: Option<Hash32>,
    pub reject_code: CohBitReject,
    pub diagnostic_hash: Hash32,
    pub policy_hash: Hash32,
    pub timestamp: Timestamp,
    pub receipt_hash: Hash32,
}

/// The CohBit v1.0: A sealed engineering atom for governed state transitions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohBit {
    // Identity
    pub version: u16,
    pub domain: DomainId,
    pub bit_id: Hash32,

    // State Transition
    pub from_state: Hash32,
    pub to_state: Hash32,
    pub action_hash: Hash32,

    // Proof/Projection
    pub projection_hash: Hash32,
    pub certificate_hash: Hash32,
    pub verifier_id: Hash32,
    pub policy_hash: Hash32,

    // Accounting (Rationals)
    pub valuation_pre: Rational64,
    pub valuation_post: Rational64,
    pub spend: Rational64,
    pub defect: Rational64,
    pub delta_hat: Rational64,
    pub authority: Rational64,

    // Routing Metrics
    pub utility: Rational64,
    pub probability_soft: Rational64,
    pub probability_exec: Rational64,

    // Ordering / Chain Linkage
    pub step_index: u64,
    pub prev_receipt_hash: Option<Hash32>,
    pub chain_digest_pre: Hash32,
    pub chain_digest_post: Hash32,

    // Verifier Result
    pub rv_status: RvStatus,

    // Cryptographic Commitment
    pub receipt_hash: Hash32,
    pub signature: Signature,
}

impl Default for CohBit {
    fn default() -> Self {
        Self {
            version: 1,
            domain: DomainId(Hash32([0; 32])),
            bit_id: Hash32([0; 32]),
            from_state: Hash32([0; 32]),
            to_state: Hash32([0; 32]),
            action_hash: Hash32([0; 32]),
            projection_hash: Hash32([0; 32]),
            certificate_hash: Hash32([0; 32]),
            verifier_id: Hash32([0; 32]),
            policy_hash: Hash32([0; 32]),
            valuation_pre: Rational64::from_integer(0),
            valuation_post: Rational64::from_integer(0),
            spend: Rational64::from_integer(0),
            defect: Rational64::from_integer(0),
            delta_hat: Rational64::from_integer(0),
            authority: Rational64::from_integer(0),
            utility: Rational64::from_integer(0),
            probability_soft: Rational64::from_integer(0),
            probability_exec: Rational64::from_integer(0),
            step_index: 0,
            prev_receipt_hash: None,
            chain_digest_pre: Hash32([0; 32]),
            chain_digest_post: Hash32([0; 32]),
            rv_status: RvStatus::Unknown,
            receipt_hash: Hash32([0; 32]),
            signature: Signature(vec![0; 64]),
        }
    }
}

impl CohBit {
    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"cohbit:v1:receipt");
        
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.domain.0.0);
        hasher.update(self.bit_id.0);
        hasher.update(self.from_state.0);
        hasher.update(self.to_state.0);
        hasher.update(self.action_hash.0);
        hasher.update(self.projection_hash.0);
        hasher.update(self.certificate_hash.0);
        hasher.update(self.verifier_id.0);
        hasher.update(self.policy_hash.0);

        let mut update_rat = |r: &Rational64| {
            let nr = r.reduced();
            hasher.update(nr.numer().to_be_bytes());
            hasher.update(nr.denom().to_be_bytes());
        };
        
        update_rat(&self.valuation_pre);
        update_rat(&self.valuation_post);
        update_rat(&self.spend);
        update_rat(&self.defect);
        update_rat(&self.delta_hat);
        update_rat(&self.authority);
        update_rat(&self.utility);
        update_rat(&self.probability_soft);
        update_rat(&self.probability_exec);

        hasher.update(self.step_index.to_be_bytes());
        if let Some(prev) = &self.prev_receipt_hash {
            hasher.update([1]);
            hasher.update(prev.0);
        } else {
            hasher.update([0]);
        }
        hasher.update(self.chain_digest_pre.0);
        hasher.update(self.chain_digest_post.0);
        hasher.update([self.rv_status as u8]);

        Hash32(hasher.finalize().into())
    }

    pub fn hash_valid(&self) -> bool {
        self.receipt_hash == self.canonical_hash()
    }

    pub fn margin(&self) -> Rational64 {
        self.valuation_pre + self.defect + self.authority - self.valuation_post - self.spend
    }

    pub fn defect_certified(&self) -> bool {
        self.defect <= self.delta_hat
    }

    pub fn budget_admissible(&self) -> bool {
        self.margin() >= Rational64::from_integer(0)
    }

    pub fn executable(&self) -> bool {
        self.hash_valid() 
            && self.defect_certified() 
            && self.budget_admissible() 
            && self.rv_status == RvStatus::Accept
    }

    pub fn identity(state_hash: Hash32, valuation: Rational64, domain: DomainId) -> Self {
        let mut bit = Self {
            version: 1,
            domain,
            from_state: state_hash,
            to_state: state_hash,
            action_hash: state_hash,
            projection_hash: state_hash,
            valuation_pre: valuation,
            valuation_post: valuation,
            spend: Rational64::from_integer(0),
            defect: Rational64::from_integer(0),
            delta_hat: Rational64::from_integer(0),
            authority: Rational64::from_integer(0),
            utility: Rational64::from_integer(0),
            probability_soft: Rational64::from_integer(1),
            probability_exec: Rational64::from_integer(1),
            rv_status: RvStatus::Accept,
            ..Default::default()
        };
        bit.receipt_hash = bit.canonical_hash();
        bit
    }
}

/// Governance Layer (PhaseLoom / NPE Layer)
pub struct CohBitLaw;

impl CohBitLaw {
    pub fn verify_probabilities(bits: &[CohBit]) -> bool {
        let sum_soft: Rational64 = bits.iter().map(|b| b.probability_soft).sum();
        let sum_exec: Rational64 = bits.iter().map(|b| b.probability_exec).sum();
        sum_soft == Rational64::from_integer(1) && sum_exec == Rational64::from_integer(1)
    }

    pub fn compute_soft_probabilities(bits: &mut [CohBit], tau: f64, beta: f64) {
        if bits.is_empty() { return; }
        let log_energies: Vec<f64> = bits.iter().map(|b| {
            let m = b.margin().to_f64().unwrap_or(0.0);
            let gate = 1.0 / (1.0 + (-beta * m).exp());
            (b.utility.to_f64().unwrap_or(0.0) / tau) + gate.ln()
        }).collect();
        let max_log = log_energies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum_exp: f64 = log_energies.iter().map(|&l| (l - max_log).exp()).sum();
        let log_z = max_log + sum_exp.ln();
        for (i, b) in bits.iter_mut().enumerate() {
            let prob = (log_energies[i] - log_z).exp();
            b.probability_soft = Rational64::approximate_float(prob).unwrap_or_else(|| Rational64::from_integer(0));
        }
    }

    pub fn compute_exec_probabilities(bits: &mut [CohBit], tau: f64) {
        if bits.is_empty() { return; }
        let log_energies: Vec<f64> = bits.iter().map(|b| {
            if b.executable() { b.utility.to_f64().unwrap_or(0.0) / tau } else { f64::NEG_INFINITY }
        }).collect();
        let max_log = log_energies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if max_log == f64::NEG_INFINITY {
            for b in bits.iter_mut() {
                if b.action_hash == b.from_state { b.probability_exec = Rational64::from_integer(1); }
                else { b.probability_exec = Rational64::from_integer(0); }
            }
            return;
        }
        let sum_exp: f64 = log_energies.iter().map(|&l| (l - max_log).exp()).sum();
        let log_z = max_log + sum_exp.ln();
        for (i, b) in bits.iter_mut().enumerate() {
            if log_energies[i] == f64::NEG_INFINITY { b.probability_exec = Rational64::from_integer(0); }
            else {
                let prob = (log_energies[i] - log_z).exp();
                b.probability_exec = Rational64::approximate_float(prob).unwrap_or_else(|| Rational64::from_integer(0));
            }
        }
    }

    pub fn verify_chain(bits: &[CohBit]) -> Result<(), CohBitReject> {
        for i in 0..bits.len() {
            let b = &bits[i];
            if !b.executable() { return Err(CohBitReject::CertificateRejected); }
            if i > 0 {
                let prev = &bits[i-1];
                if b.from_state != prev.to_state { return Err(CohBitReject::StateHashMismatch); }
                if b.prev_receipt_hash != Some(prev.receipt_hash) { return Err(CohBitReject::PreviousReceiptMismatch); }
                if b.step_index != prev.step_index + 1 { return Err(CohBitReject::ChainIndexMismatch); }
                if b.chain_digest_pre != prev.chain_digest_post { return Err(CohBitReject::ChainDigestMismatch); }
            }
            let expected_digest = b.chain_digest_pre.combine_tagged("cohbit:v1:chain", &b.receipt_hash);
            if b.chain_digest_post != expected_digest { return Err(CohBitReject::ChainDigestMismatch); }
        }
        Ok(())
    }
}

pub struct CohBitThermodynamics;

impl CohBitThermodynamics {
    pub fn soft_entropy(bits: &[CohBit]) -> f64 {
        bits.iter().map(|b| {
            let p = b.probability_soft.to_f64().unwrap_or(0.0);
            if p > 1e-15 { -p * p.ln() } else { 0.0 }
        }).sum()
    }

    pub fn exec_entropy(bits: &[CohBit]) -> f64 {
        bits.iter().map(|b| {
            let p = b.probability_exec.to_f64().unwrap_or(0.0);
            if p > 1e-15 { -p * p.ln() } else { 0.0 }
        }).sum()
    }

    pub fn enforcement_loss(bits: &[CohBit]) -> f64 {
        Self::soft_entropy(bits) - Self::exec_entropy(bits)
    }
}
