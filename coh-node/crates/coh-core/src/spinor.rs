use crate::cohbit::CohBit;
use crate::atom::CohAtom;
use crate::types::{Hash32, DomainId, Signature};
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use sha2::Digest;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CohSpinorReject {
    #[error("Non-canonical encoding")]
    NonCanonicalEncoding,
    #[error("Bad spinor hash")]
    BadSpinorHash,
    #[error("Bad signature")]
    BadSignature,
    #[error("Atom mismatch")]
    AtomMismatch,
    #[error("State mismatch")]
    StateMismatch,
    #[error("Basis mismatch")]
    BasisMismatch,
    #[error("Transform mismatch")]
    TransformMismatch,
    #[error("Negative amplitude")]
    NegativeAmplitude,
    #[error("Negative norm")]
    NegativeNorm,
    #[error("Phase out of range")]
    PhaseOutOfRange,
    #[error("Alignment out of range")]
    AlignmentOutOfRange,
    #[error("Instability phase out of range")]
    InstabilityPhaseOutOfRange,
    #[error("Norm not preserved")]
    NormNotPreserved,
    #[error("Unauthorized inversion")]
    UnauthorizedInversion,
    #[error("Phase continuity break")]
    PhaseContinuityBreak,
    #[error("Unsupported version")]
    UnsupportedVersion,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Orientation {
    Forward = 0,
    Reverse = 1,
    Neutral = 2,
    Mixed = 3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Parity {
    Even = 0,
    Odd = 1,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpinContext {
    pub k1_amplification: Rational64,
    pub k2_decay: Rational64,
    pub tau_drift: Rational64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpinInstability {
    Stable,
    PhaseDrift,
    Decoherence,
    SpinFlip,
    Turbulence,
}

/// The CohSpinor v1.0: Oriented internal state of a CohAtom.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohSpinor {
    pub version: u16,
    pub domain: DomainId,

    pub atom_hash: Hash32,
    pub state_hash: Hash32,
    pub frame_hash: Hash32,

    pub amplitude: Rational64,
    pub phase_num: Rational64,
    pub phase_den: Rational64,

    pub orientation: Orientation,
    pub parity: Parity,

    pub basis_hash: Hash32,
    pub transform_hash: Hash32,

    pub norm: Rational64,
    pub coherence_alignment: Rational64,
    pub instability_phase: Rational64,

    pub spinor_hash: Hash32,
    pub signature: Signature,
}

impl Default for CohSpinor {
    fn default() -> Self {
        Self {
            version: 1,
            domain: DomainId(Hash32([0; 32])), // fixture_only: allow_mock
            atom_hash: Hash32([0; 32]), // fixture_only: allow_mock
            state_hash: Hash32([0; 32]), // fixture_only: allow_mock
            frame_hash: Hash32([0; 32]), // fixture_only: allow_mock
            amplitude: Rational64::from_integer(0),
            phase_num: Rational64::from_integer(0),
            phase_den: Rational64::from_integer(1),
            orientation: Orientation::Neutral,
            parity: Parity::Even,
            basis_hash: Hash32([0; 32]), // fixture_only: allow_mock
            transform_hash: Hash32([0; 32]), // fixture_only: allow_mock
            norm: Rational64::from_integer(0),
            coherence_alignment: Rational64::from_integer(0),
            instability_phase: Rational64::from_integer(0),
            spinor_hash: Hash32([0; 32]), // fixture_only: allow_mock
            signature: Signature(vec![0; 64]), // fixture_only: allow_mock
        }
    }
}

impl CohSpinor {
    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"cohspinor:v1:state");
        
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.domain.0.0);
        hasher.update(self.atom_hash.0);
        hasher.update(self.state_hash.0);
        hasher.update(self.frame_hash.0);

        let update_rat = |hasher: &mut sha2::Sha256, r: &Rational64| {
            let nr = r.reduced();
            hasher.update(nr.numer().to_be_bytes());
            hasher.update(nr.denom().to_be_bytes());
        };
        
        update_rat(&mut hasher, &self.amplitude);
        update_rat(&mut hasher, &self.phase_num);
        update_rat(&mut hasher, &self.phase_den);

        hasher.update([self.orientation as u8]);
        hasher.update([self.parity as u8]);

        hasher.update(self.basis_hash.0);
        hasher.update(self.transform_hash.0);

        update_rat(&mut hasher, &self.norm);
        update_rat(&mut hasher, &self.coherence_alignment);
        update_rat(&mut hasher, &self.instability_phase);

        Hash32(hasher.finalize().into())
    }

    pub fn hash_valid(&self) -> bool {
        self.spinor_hash == self.canonical_hash()
    }

    pub fn alignment(&self, bit: &CohBit) -> Rational64 {
        let numer = bit.utility - bit.defect;
        let denom = if bit.utility > Rational64::from_integer(1) { bit.utility } else { Rational64::from_integer(1) };
        (numer / denom).reduced()
    }

    pub fn phase_shift(&self, bit: &CohBit) -> Rational64 {
        let align = self.alignment(bit);
        (Rational64::from_integer(1) - align).reduced()
    }

    pub fn amplitude_update(&self, bit: &CohBit, ctx: &SpinContext) -> Rational64 {
        let align = self.alignment(bit);
        let margin = bit.margin();
        let term1 = ctx.k1_amplification * margin;
        let term2 = ctx.k2_decay * (Rational64::from_integer(1) - align);
        (term1 - term2).reduced()
    }

    pub fn evolve(&self, bit: &CohBit, ctx: &SpinContext) -> Result<Self, CohSpinorReject> {
        if !bit.executable() {
            return Ok(self.clone());
        }

        let mut next = self.clone();
        
        let delta_rho = self.amplitude_update(bit, ctx);
        let mut new_norm = self.norm + delta_rho;
        
        let max_norm = self.norm + bit.delta_hat;
        if new_norm > max_norm { new_norm = max_norm; }
        if new_norm < Rational64::from_integer(0) { new_norm = Rational64::from_integer(0); }
        next.norm = new_norm;
        next.amplitude = new_norm;

        let delta_theta = self.phase_shift(bit);
        let current_phase = self.phase_num / self.phase_den;
        let next_phase = (current_phase + delta_theta).reduced();
        next.phase_num = Rational64::from_integer(*next_phase.numer());
        next.phase_den = Rational64::from_integer(*next_phase.denom());

        let align = self.alignment(bit);
        if align < Rational64::new(-1, 2) {
            next.orientation = match self.orientation {
                Orientation::Forward => Orientation::Reverse,
                Orientation::Reverse => Orientation::Forward,
                _ => self.orientation,
            };
        }

        next.spinor_hash = next.canonical_hash();
        Ok(next)
    }

    pub fn detect_instability(&self) -> SpinInstability {
        if self.norm < Rational64::new(1, 10) { return SpinInstability::Decoherence; }
        if self.instability_phase > Rational64::new(8, 10) { return SpinInstability::Turbulence; }
        SpinInstability::Stable
    }

    pub fn validity_checks(&self) -> Result<(), CohSpinorReject> {
        if self.amplitude < Rational64::from_integer(0) { return Err(CohSpinorReject::NegativeAmplitude); }
        if self.norm < Rational64::from_integer(0) { return Err(CohSpinorReject::NegativeNorm); }
        if self.coherence_alignment < Rational64::from_integer(0) || self.coherence_alignment > Rational64::from_integer(1) { return Err(CohSpinorReject::AlignmentOutOfRange); }
        if self.instability_phase < Rational64::from_integer(0) || self.instability_phase > Rational64::from_integer(1) { return Err(CohSpinorReject::InstabilityPhaseOutOfRange); }
        if self.phase_den == Rational64::from_integer(0) { return Err(CohSpinorReject::PhaseOutOfRange); }
        Ok(())
    }

    pub fn compatible_with_atom(&self, atom: &CohAtom) -> Result<(), CohSpinorReject> {
        if self.atom_hash != atom.atom_hash { return Err(CohSpinorReject::AtomMismatch); }
        if self.state_hash != atom.final_state { return Err(CohSpinorReject::StateMismatch); }
        Ok(())
    }

    pub fn phase_alignment_with_bit(&self, bit: &CohBit) -> Result<Rational64, CohSpinorReject> {
        let alignment = self.coherence_alignment * bit.utility;
        Ok(alignment.reduced())
    }

    pub fn route_candidates<'a>(&self, bits: &'a [CohBit]) -> Result<Vec<(&'a CohBit, Rational64)>, CohSpinorReject> {
        let mut weighted = vec![];
        for bit in bits {
            if bit.executable() {
                let weight = self.phase_alignment_with_bit(bit)?;
                weighted.push((bit, weight));
            }
        }
        weighted.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(weighted)
    }
}
