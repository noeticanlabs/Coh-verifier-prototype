use crate::cohbit::{CohBit, CohBitReject};
use crate::types::{Hash32, DomainId, Signature};
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use num_traits::ToPrimitive;
use sha2::Digest;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CohAtomReject {
    #[error("Non-canonical encoding")]
    NonCanonicalEncoding,
    #[error("Bad atom hash")]
    BadAtomHash,
    #[error("Bad signature")]
    BadSignature,
    #[error("Empty bits")]
    EmptyBits,
    #[error("Bit rejected")]
    BitRejected(CohBitReject),
    #[error("Initial state mismatch")]
    InitialStateMismatch,
    #[error("Final state mismatch")]
    FinalStateMismatch,
    #[error("State continuity break")]
    StateContinuityBreak,
    #[error("Receipt continuity break")]
    ReceiptContinuityBreak,
    #[error("Chain digest break")]
    ChainDigestBreak,
    #[error("Step index break")]
    StepIndexBreak,
    #[error("Domain mismatch")]
    DomainMismatch,
    #[error("Policy mismatch")]
    PolicyMismatch,
    #[error("Verifier mismatch")]
    VerifierMismatch,
    #[error("Cumulative spend mismatch")]
    CumulativeSpendMismatch,
    #[error("Cumulative defect mismatch")]
    CumulativeDefectMismatch,
    #[error("Cumulative delta-hat mismatch")]
    CumulativeDeltaHatMismatch,
    #[error("Cumulative authority mismatch")]
    CumulativeAuthorityMismatch,
    #[error("Defect envelope exceeded")]
    DefectEnvelopeExceeded,
    #[error("Negative total margin")]
    NegativeTotalMargin,
    #[error("Authority cap exceeded")]
    AuthorityCapExceeded,
    #[error("Authority double-spend")]
    AuthorityDoubleSpend,
    #[error("Execution replay mismatch")]
    ExecutionReplayMismatch,
    #[error("Parallel conflict")]
    ParallelConflict,
    #[error("Unsupported version")]
    UnsupportedVersion,
    #[error("Summary missing compression certificate")]
    SummaryMissingCompressionCertificate,
    #[error("Mock certificate rejected")] // fixture_only: allow_mock
    MockCertificateRejected, // fixture_only: allow_mock
    #[error("Placeholder hash rejected")] // fixture_only: allow_mock
    PlaceholderHashRejected, // fixture_only: allow_mock
    #[error("Fixture data in production")] // fixture_only: allow_mock
    FixtureDataInProduction, // fixture_only: allow_mock
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CouplingWitness {
    pub independence_hash: Hash32,
    pub conflict_set_hash: Hash32,
    pub coupling_defect: Rational64,
    pub coupling_spend: Rational64,
}

impl CouplingWitness {
    pub fn hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.independence_hash.0);
        hasher.update(self.conflict_set_hash.0);
        hasher.update(self.coupling_defect.reduced().numer().to_be_bytes());
        hasher.update(self.coupling_defect.reduced().denom().to_be_bytes());
        hasher.update(self.coupling_spend.reduced().numer().to_be_bytes());
        hasher.update(self.coupling_spend.reduced().denom().to_be_bytes());
        Hash32(hasher.finalize().into())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AtomKind {
    ExecutableTrajectory,
    SummaryTrajectory,
    Identity,
}

/// The CohAtom v1.1: A closed, self-consistent transition complex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohAtom {
    pub version: u16,
    pub kind: AtomKind, // NEW
    pub domain: DomainId,
    pub atom_id: Hash32,

    pub initial_state: Hash32,
    pub final_state: Hash32,

    pub bits: Vec<CohBit>,

    pub cumulative_spend: Rational64,
    pub cumulative_defect: Rational64,
    pub cumulative_delta_hat: Rational64,
    pub cumulative_authority: Rational64,

    pub margin_total: Rational64,

    pub policy_hash: Hash32,
    pub verifier_id: Hash32,

    pub atom_digest_pre: Hash32,
    pub atom_digest_post: Hash32,

    pub atom_hash: Hash32,
    pub signature: Signature,
    pub compression_certificate: Option<Hash32>, // NEW
    
    // Boundary caching for O(1) summary verification
    pub initial_valuation: Option<Rational64>,
    pub final_valuation: Option<Rational64>,
}

impl Default for CohAtom {
    fn default() -> Self {
        // fixture_only: allow_mock
        Self {
            version: 1,
            kind: AtomKind::ExecutableTrajectory,
            domain: DomainId(Hash32([0; 32])), // fixture_only: allow_mock
            atom_id: Hash32([0; 32]), // fixture_only: allow_mock
            initial_state: Hash32([0; 32]), // fixture_only: allow_mock
            final_state: Hash32([0; 32]), // fixture_only: allow_mock
            bits: vec![],
            cumulative_spend: Rational64::from_integer(0),
            cumulative_defect: Rational64::from_integer(0),
            cumulative_delta_hat: Rational64::from_integer(0),
            cumulative_authority: Rational64::from_integer(0),
            margin_total: Rational64::from_integer(0),
            policy_hash: Hash32([0; 32]), // fixture_only: allow_mock
            verifier_id: Hash32([0; 32]), // fixture_only: allow_mock
            atom_digest_pre: Hash32([0; 32]), // fixture_only: allow_mock
            atom_digest_post: Hash32([0; 32]), // fixture_only: allow_mock
            atom_hash: Hash32([0; 32]), // fixture_only: allow_mock
            signature: Signature(vec![0; 64]), // fixture_only: allow_mock
            compression_certificate: None,
            initial_valuation: None,
            final_valuation: None,
        }
    }
}

impl CohAtom {
    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"cohatom:v1:receipt");
        
        hasher.update(self.version.to_be_bytes());
        hasher.update([self.kind as u8]);
        hasher.update(self.domain.0.0);
        hasher.update(self.atom_id.0);
        hasher.update(self.initial_state.0);
        hasher.update(self.final_state.0);

        for bit in &self.bits {
            hasher.update(bit.receipt_hash.0);
        }

        let mut update_rat = |r: &Rational64| {
            let nr = r.reduced();
            hasher.update(nr.numer().to_be_bytes());
            hasher.update(nr.denom().to_be_bytes());
        };
        
        update_rat(&self.cumulative_spend);
        update_rat(&self.cumulative_defect);
        update_rat(&self.cumulative_delta_hat);
        update_rat(&self.cumulative_authority);
        update_rat(&self.margin_total);

        hasher.update(self.policy_hash.0);
        hasher.update(self.verifier_id.0);
        hasher.update(self.atom_digest_pre.0);
        hasher.update(self.atom_digest_post.0);
        
        if let Some(ref cert) = self.compression_certificate {
            hasher.update([1]);
            hasher.update(cert.0);
        } else {
            hasher.update([0]);
        }

        if let Some(val) = self.initial_valuation {
            hasher.update([1]);
            hasher.update(val.reduced().numer().to_be_bytes());
            hasher.update(val.reduced().denom().to_be_bytes());
        } else {
            hasher.update([0]);
        }

        if let Some(val) = self.final_valuation {
            hasher.update([1]);
            hasher.update(val.reduced().numer().to_be_bytes());
            hasher.update(val.reduced().denom().to_be_bytes());
        } else {
            hasher.update([0]);
        }

        Hash32(hasher.finalize().into())
    }

    pub fn hash_valid(&self) -> bool {
        self.atom_hash == self.canonical_hash()
    }

    pub fn structural_continuity_valid(&self) -> Result<(), CohAtomReject> {
        if self.kind == AtomKind::SummaryTrajectory {
            // Summaries rely on the compression certificate and boundary caches
            if self.compression_certificate.is_none() { return Err(CohAtomReject::SummaryMissingCompressionCertificate); }
            return Ok(());
        }

        if self.bits.is_empty() {
            return if self.initial_state == self.final_state { Ok(()) } else { Err(CohAtomReject::EmptyBits) };
        }
        // ... (existing bit-level checks)

        if self.bits[0].from_state != self.initial_state { return Err(CohAtomReject::InitialStateMismatch); }
        if self.bits.last().unwrap().to_state != self.final_state { return Err(CohAtomReject::FinalStateMismatch); }

        for i in 0..self.bits.len() {
            let b = &self.bits[i];
            if !b.structural_executable() { return Err(CohAtomReject::BitRejected(CohBitReject::CertificateRejected)); }
            if i > 0 {
                let prev = &self.bits[i-1];
                if b.from_state != prev.to_state { return Err(CohAtomReject::StateContinuityBreak); }
                if b.prev_receipt_hash != Some(prev.receipt_hash) { return Err(CohAtomReject::ReceiptContinuityBreak); }
                if b.step_index != prev.step_index + 1 { return Err(CohAtomReject::StepIndexBreak); }
                if b.chain_digest_pre != prev.chain_digest_post { return Err(CohAtomReject::ChainDigestBreak); }
            }
        }
        Ok(())
    }

    pub fn continuity_valid(&self) -> Result<(), CohAtomReject> {
        if self.kind == AtomKind::SummaryTrajectory {
            return self.structural_continuity_valid();
        }
        self.structural_continuity_valid()?;
        for b in &self.bits {
            if !b.executable() { return Err(CohAtomReject::BitRejected(CohBitReject::NegativeMargin)); }
        }
        Ok(())
    }

    pub fn recompute_metrics(&self) -> (Rational64, Rational64, Rational64, Rational64) {
        let mut spend = Rational64::from_integer(0);
        let mut defect = Rational64::from_integer(0);
        let mut delta_hat = Rational64::from_integer(0);
        let mut authority = Rational64::from_integer(0);

        for b in &self.bits {
            spend += b.spend;
            defect += b.defect;
            delta_hat += b.delta_hat;
            authority += b.authority;
        }
        (spend, defect, delta_hat, authority)
    }

    pub fn metrics_valid(&self) -> Result<(), CohAtomReject> {
        if self.kind == AtomKind::SummaryTrajectory {
            // A summary is valid if its cumulative metrics don't exceed the envelope
            if self.cumulative_defect > self.cumulative_delta_hat { return Err(CohAtomReject::DefectEnvelopeExceeded); }
            return Ok(());
        }

        let (spend, defect, delta_hat, authority) = self.recompute_metrics();

        if spend != self.cumulative_spend { return Err(CohAtomReject::CumulativeSpendMismatch); }
        if defect != self.cumulative_defect { return Err(CohAtomReject::CumulativeDefectMismatch); }
        if delta_hat != self.cumulative_delta_hat { return Err(CohAtomReject::CumulativeDeltaHatMismatch); }
        if authority != self.cumulative_authority { return Err(CohAtomReject::CumulativeAuthorityMismatch); }
        if defect > delta_hat { return Err(CohAtomReject::DefectEnvelopeExceeded); }
        Ok(())
    }

    pub fn budget_valid(&self, v_pre: Rational64, v_post: Rational64) -> Result<(), CohAtomReject> {
        let expected_margin = v_pre + self.cumulative_defect + self.cumulative_authority - v_post - self.cumulative_spend;
        if expected_margin != self.margin_total || self.margin_total < Rational64::from_integer(0) {
            return Err(CohAtomReject::NegativeTotalMargin);
        }
        Ok(())
    }

    pub fn structural_executable(&self) -> bool {
        if self.atom_id.0 == [0; 32] || self.atom_hash.0 == [0; 32] {
            return false;
        }

        if let Some(cert) = self.compression_certificate {
            if cert.0 == [0xCC; 32] || cert.0 == [0xAA; 32] { // fixture_only: allow_mock
                return false;
            }
        }

        self.hash_valid() 
            && self.structural_continuity_valid().is_ok() 
            && self.metrics_valid().is_ok()
    }

    pub fn retrieval_valid(&self) -> bool {
        match self.kind {
            AtomKind::ExecutableTrajectory => self.structural_executable(),
            AtomKind::Identity => self.structural_executable(),
            AtomKind::SummaryTrajectory => {
                self.compression_certificate.is_some() && self.structural_executable()
            }
        }
    }

    pub fn mutation_valid(&self) -> bool {
        match self.kind {
            AtomKind::ExecutableTrajectory | AtomKind::Identity => {
                self.retrieval_valid() 
                    && self.continuity_valid().is_ok()
                    && self.budget_valid(self.initial_valuation(), self.final_valuation()).is_ok()
            }
            AtomKind::SummaryTrajectory => false, // Summaries are not for mutation
        }
    }

    pub fn executable(&self) -> bool {
        self.mutation_valid()
    }

    /// The Refinery Operation: Transition from ExecutableTrajectory to SummaryTrajectory.
    /// This is a one-way transformation that minimizes footprint while preserving validity.
    pub fn compress(&mut self) -> Result<(), CohAtomReject> {
        if self.kind != AtomKind::ExecutableTrajectory {
            return Err(CohAtomReject::UnsupportedVersion); // Cannot compress a non-executable atom
        }

        // 1. Verify before compression
        if !self.executable() {
            return Err(CohAtomReject::BitRejected(CohBitReject::CertificateRejected));
        }

        // 2. Cache boundary valuations for O(1) budget verification
        self.initial_valuation = Some(self.initial_valuation());
        self.final_valuation = Some(self.final_valuation());

        // 3. Compute Merkle Root of bits
        let leaves: Vec<Hash32> = self.bits.iter().map(|b| b.receipt_hash).collect();
        self.compression_certificate = Some(crate::merkle::build_merkle_root(&leaves));

        // 4. Transform to Summary
        self.kind = AtomKind::SummaryTrajectory;
        self.bits.clear();
        self.bits.shrink_to_fit(); // Release heap memory

        // 5. Seal the atom
        self.atom_hash = self.canonical_hash();
        Ok(())
    }

    pub fn initial_valuation(&self) -> Rational64 {
        if let Some(val) = self.initial_valuation { return val; }
        self.bits.first().map(|b| b.valuation_pre).unwrap_or(Rational64::from_integer(0))
    }

    pub fn final_valuation(&self) -> Rational64 {
        if let Some(val) = self.final_valuation { return val; }
        self.bits.last().map(|b| b.valuation_post).unwrap_or(Rational64::from_integer(0))
    }

    pub fn identity_atom(state: Hash32, valuation: Rational64, domain: DomainId) -> Self {
        let id_bit = CohBit::identity_atom(state, valuation, domain);
        let mut atom = Self {
            kind: AtomKind::Identity,
            domain,
            atom_id: Hash32([0x22; 32]), // fixture_only: allow_mock
            initial_state: state,
            final_state: state,
            bits: vec![id_bit],
            ..Default::default()
        };
        atom.atom_hash = atom.canonical_hash();
        atom
    }

    pub fn certified_identity(
        state: Hash32, 
        valuation: Rational64, 
        domain: DomainId,
        verifier_id: Hash32,
        policy_hash: Hash32,
    ) -> Self {
        let id_bit = CohBit::certified_identity(state, valuation, domain, verifier_id, policy_hash, Hash32::tagged_hash("cohbit:v1:id", &[state.0, verifier_id.0, policy_hash.0]));
        let mut atom = Self {
            kind: AtomKind::Identity,
            domain,
            initial_state: state,
            final_state: state,
            bits: vec![id_bit],
            verifier_id,
            policy_hash,
            atom_id: Hash32::tagged_hash("cohatom:v1:id", &[state.0, verifier_id.0, policy_hash.0]),
            ..Default::default()
        };
        atom.atom_hash = atom.canonical_hash();
        atom
    }
}

/// Coh Atom Geometry Metrics (The Lab Coat)
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AtomGeometry {
    pub distance: Rational64,
    pub curvature: f64,
    pub ricci_scalar: f64,
}

/// Coh Atom Metabolism
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AtomMetabolism {
    pub budget: Rational64,
    pub refresh: Rational64,
}

/// The Coh Governor: The runtime engine that manages state evolution.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CohGovernor {
    pub state_hash: Hash32,
    pub valuation: Rational64,
    pub geometry: AtomGeometry,
    pub metabolism: AtomMetabolism,
}

impl CohGovernor {
    pub fn compute_action(&self, bit: &CohBit, lambda: f64, gauge_curvature: f64) -> f64 {
        let delta_hat = bit.delta_hat.to_f64().unwrap_or(0.0);
        let f_exec = bit.utility.to_f64().unwrap_or(0.0);
        let ricci = self.geometry.ricci_scalar;
        let r_coh = ricci + gauge_curvature;
        let u_refresh = self.metabolism.refresh.to_f64().unwrap_or(0.0);
        delta_hat - f_exec + (lambda * r_coh) - u_refresh
    }

    pub fn select_optimal_bit<'a>(&self, bits: &'a [CohBit], lambda: f64, gauge_curvature: f64) -> Option<&'a CohBit> {
        bits.iter()
            .filter(|b| b.executable())
            .min_by(|a, b| {
                let ja = self.compute_action(a, lambda, gauge_curvature);
                let jb = self.compute_action(b, lambda, gauge_curvature);
                ja.partial_cmp(&jb).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn update_metabolism(&mut self, bit: &CohBit) {
        self.metabolism.budget = self.metabolism.budget + self.metabolism.refresh - bit.spend;
    }

    pub fn evolve(&mut self, bit: &CohBit) -> bool {
        if !bit.executable() { return false; }
        let lhs = bit.valuation_post + bit.spend;
        let rhs = self.valuation + bit.defect + self.metabolism.refresh;
        if lhs > rhs { return false; }
        self.state_hash = bit.to_state;
        self.valuation = bit.valuation_post;
        self.update_metabolism(bit);
        true
    }
}

#[cfg(test)]
mod refinery_tests;
