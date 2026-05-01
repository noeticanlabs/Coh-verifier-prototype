// use crate::atom::CohAtom;
use crate::atom::CohAtom;
use crate::types::{Hash32, DomainId, Signature};
use crate::decoherence::{
    DecoherenceContext, DecoherenceResult, DecoherenceReject, DecoherenceState,
    DecoherenceCause, DecoherenceCertificate, QuarantineReceipt, AuthorityGrant,
    DecoherenceMode, MonogamyState, EntanglementMode, EntanglementContext
};
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use num_traits::Zero;
use sha2::Digest;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum EntanglementReject {
    #[error("Atom validity failure (E2)")]
    AtomInvalid,
    #[error("Domain/Policy mismatch (E3)")]
    ContextMismatch,
    #[error("Negative joint margin (E4)")]
    NegativeJointMargin,
    #[error("Joint defect bound exceeded (E5)")]
    DefectBoundExceeded,
    #[error("Joint authority bound exceeded (E6)")]
    AuthorityBoundExceeded,
    #[error("Monogamy scope reuse detected (E7)")]
    MonogamyViolation,
    #[error("Entanglement hash mismatch")]
    EntanglementHashMismatch,
    #[error("Joint margin mismatch")]
    JointMarginMismatch,
    #[error("Coupling witness invalid (E9)")]
    WitnessInvalid,
    #[error("Decoherence: atoms failed independent validation (E11)")]
    DecoherenceFailure,
}

pub struct MonogamyRegistry {
    pub keys: std::collections::HashMap<Hash32, MonogamyState>,
}

impl MonogamyRegistry {
    pub fn check(&self, key: &Hash32) -> Result<(), EntanglementReject> {
        if let Some(state) = self.keys.get(key) {
            if *state == MonogamyState::Active {
                return Err(EntanglementReject::MonogamyViolation);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CouplingWitnessKind {
    FixtureOnly,
    HeuristicCorrelation,
    CertifiedNonSeparability,
}

/// The EntangledCohAtom v2.4: A non-separable multi-trajectory complex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntangledCohAtom {
    pub atoms: Vec<CohAtom>,

    pub shared_defect: Rational64,
    pub shared_delta_hat: Rational64,
    pub shared_authority: Rational64,
    pub shared_authority_cap: Rational64,

    pub joint_margin: Rational64,

    pub domain_id: DomainId,
    pub policy_hash: Hash32,
    pub monogamy_scope: Hash32,
    
    pub witness_kind: CouplingWitnessKind,
    pub coupling_witness: Hash32,

    pub entanglement_hash: Hash32,
}

impl EntangledCohAtom {
    pub fn new(
        atoms: Vec<CohAtom>,
        shared_defect: Rational64,
        shared_delta_hat: Rational64,
        shared_authority: Rational64,
        shared_authority_cap: Rational64,
        domain_id: DomainId,
        policy_hash: Hash32,
        monogamy_scope: Hash32,
        witness_kind: CouplingWitnessKind,
        coupling_witness: Hash32,
    ) -> Self {
        let mut e = Self {
            atoms,
            shared_defect,
            shared_delta_hat,
            shared_authority,
            shared_authority_cap,
            joint_margin: Rational64::from_integer(0),
            domain_id,
            policy_hash,
            monogamy_scope,
            witness_kind,
            coupling_witness,
            entanglement_hash: Hash32([0; 32]), // fixture_only: allow_mock
        };
        e.joint_margin = e.calculate_joint_margin();
        e.entanglement_hash = e.canonical_hash();
        e
    }

    pub fn calculate_joint_margin(&self) -> Rational64 {
        let sum_valuation_pre: Rational64 = self.atoms.iter().map(|a| a.initial_valuation()).sum();
        let sum_valuation_post: Rational64 = self.atoms.iter().map(|a| a.final_valuation()).sum();
        let sum_spend: Rational64 = self.atoms.iter().map(|a| a.cumulative_spend).sum();

        // Joint Admissibility Law:
        // V_joint(out) + sum(spend) <= V_joint(in) + delta_shared + alpha_shared
        // margin = V_joint(in) + delta_shared + alpha_shared - (V_joint(out) + sum(spend))
        (sum_valuation_pre + self.shared_defect + self.shared_authority) - (sum_valuation_post + sum_spend)
    }

    pub fn monogamy_key(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.domain_id.0.0);
        hasher.update(self.policy_hash.0);
        hasher.update(self.monogamy_scope.0);
        Hash32(hasher.finalize().into())
    }

    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"cohentanglement:v2.4");
        for atom in &self.atoms {
            hasher.update(atom.atom_hash.0);
        }
        
        let mut update_rat = |r: &Rational64| {
            hasher.update(r.reduced().numer().to_be_bytes());
            hasher.update(r.reduced().denom().to_be_bytes());
        };

        update_rat(&self.shared_defect);
        update_rat(&self.shared_delta_hat);
        update_rat(&self.shared_authority);
        update_rat(&self.shared_authority_cap);
        
        hasher.update(self.domain_id.0.0);
        hasher.update(self.policy_hash.0);
        hasher.update(self.monogamy_scope.0);
        hasher.update([self.witness_kind as u8]);
        hasher.update(self.coupling_witness.0);
        
        Hash32(hasher.finalize().into())
    }

    pub fn verify_for_creation(
        &self, 
        registry: &MonogamyRegistry,
        ctx: &EntanglementContext,
    ) -> Result<(), EntanglementReject> {
        self.verify_base(ctx)?;
        
        // E7: Monogamy Key creation check
        registry.check(&self.monogamy_key())?;
        
        Ok(())
    }

    pub fn verify_existing_registered(
        &self,
        registry: &MonogamyRegistry,
        ctx: &EntanglementContext,
    ) -> Result<(), EntanglementReject> {
        self.verify_base(ctx)?;
        
        // Ensure it IS registered
        let key = self.monogamy_key();
        if !registry.keys.contains_key(&key) {
            return Err(EntanglementReject::MonogamyViolation);
        }
        
        Ok(())
    }

    fn verify_base(&self, ctx: &EntanglementContext) -> Result<(), EntanglementReject> {
        // Hash validation
        if self.entanglement_hash != self.canonical_hash() {
            return Err(EntanglementReject::EntanglementHashMismatch);
        }

        // Joint Margin Recomputation
        let expected_margin = self.calculate_joint_margin();
        if self.joint_margin != expected_margin {
            return Err(EntanglementReject::JointMarginMismatch);
        }

        // E2: Atom Structural Validity
        for atom in &self.atoms {
            if !atom.structural_executable() { return Err(EntanglementReject::AtomInvalid); }
        }

        // E3: Domain/Policy Compatibility
        if self.domain_id != ctx.domain_id || self.policy_hash != ctx.policy_hash {
            return Err(EntanglementReject::ContextMismatch);
        }
        for atom in &self.atoms {
            if atom.domain != self.domain_id || atom.policy_hash != self.policy_hash {
                return Err(EntanglementReject::ContextMismatch);
            }
        }

        // E4: Joint Margin
        if self.joint_margin < Rational64::from_integer(0) {
            return Err(EntanglementReject::NegativeJointMargin);
        }

        // E5: Joint Defect Bound
        if self.shared_defect > self.shared_delta_hat {
            return Err(EntanglementReject::DefectBoundExceeded);
        }

        // E6: Joint Authority Bound
        if self.shared_authority > self.shared_authority_cap {
            return Err(EntanglementReject::AuthorityBoundExceeded);
        }

        // E9: Witness Validity / Mode Guard
        if self.witness_kind == CouplingWitnessKind::FixtureOnly && ctx.mode == EntanglementMode::Production {
            return Err(EntanglementReject::WitnessInvalid);
        }
        if self.coupling_witness.0 == [0; 32] {
            return Err(EntanglementReject::WitnessInvalid);
        }
        Ok(())
    }

    pub fn verify(
        &self, 
        active_monogamy_keys: &[Hash32], // Legacy compat
        is_production: bool
    ) -> Result<(), EntanglementReject> {
        let mode = if is_production { EntanglementMode::Production } else { EntanglementMode::Heuristic };
        let ctx = EntanglementContext {
            mode,
            domain_id: self.domain_id,
            policy_hash: self.policy_hash,
        };
        
        // Manual key check (legacy)
        let key = self.monogamy_key();
        if active_monogamy_keys.contains(&key) {
            return Err(EntanglementReject::MonogamyViolation);
        }
        
        self.verify_base(&ctx)
    }

    /// E11: Decoherence Check
    /// Verifies if atoms can survive independently if the coupling breaks.
    pub fn verify_decoherence(&self) -> Result<(), EntanglementReject> {
        for atom in &self.atoms {
            if !atom.executable() {
                return Err(EntanglementReject::DecoherenceFailure);
            }
        }
        Ok(())
    }

    pub fn local_margins_without_shared_budget(&self) -> Vec<Rational64> {
        self.atoms
            .iter()
            .map(|atom| {
                atom.initial_valuation()
                    + atom.cumulative_defect
                    + atom.cumulative_authority
                    - atom.final_valuation()
                    - atom.cumulative_spend
            })
            .collect()
    }

    pub fn decohere(
        &self,
        ctx: &DecoherenceContext,
        grants: &[AuthorityGrant],
        cause: DecoherenceCause,
        registry: &MonogamyRegistry,
    ) -> Result<DecoherenceResult, DecoherenceReject> {
        // D2: Original entangled complex must be valid before decoherence
        let e_ctx = EntanglementContext {
            mode: ctx.entanglement_mode,
            domain_id: self.domain_id,
            policy_hash: self.policy_hash,
        };
        self.verify_existing_registered(registry, &e_ctx)
            .map_err(|_| DecoherenceReject::InvalidEntanglement)?;

        // D10: Split participants inherit original domain and policy
        if ctx.domain_id != self.domain_id { return Err(DecoherenceReject::DomainMismatch); }
        if ctx.policy_hash != self.policy_hash { return Err(DecoherenceReject::PolicyMismatch); }

        let local_margins = self.local_margins_without_shared_budget();

        // D7: Hard split requires every participant to be locally admissible
        if local_margins.iter().all(|m| *m >= Rational64::zero()) {
            return self.hard_split(ctx, &local_margins, cause);
        }

        // D8: Assisted split requires new separately receipted authority
        if ctx.allow_assisted_split && ctx.mode == DecoherenceMode::AssistedSplit {
            if let Some(result) = self.try_assisted_split(ctx, grants, &local_margins, cause)? {
                return Ok(result);
            }
        }

        // D9: If any participant remains local-invalid, quarantine is mandatory
        self.quarantine(ctx, &local_margins, cause)
    }

    fn hard_split(
        &self,
        ctx: &DecoherenceContext,
        local_margins: &[Rational64],
        cause: DecoherenceCause,
    ) -> Result<DecoherenceResult, DecoherenceReject> {
        let split_witness = self.compute_hard_split_witness_hash(local_margins, cause);
        let cert = self.create_certificate(ctx, local_margins, cause, split_witness);
        Ok(DecoherenceResult {
            state: DecoherenceState::SplitCertified,
            certificate: Some(cert),
            local_atoms: self.atoms.clone(),
            quarantine_receipt: None,
        })
    }

    fn try_assisted_split(
        &self,
        ctx: &DecoherenceContext,
        grants: &[AuthorityGrant],
        local_margins: &[Rational64],
        cause: DecoherenceCause,
    ) -> Result<Option<DecoherenceResult>, DecoherenceReject> {
        let mut adjusted_margins = local_margins.to_vec();
        let mut updated_atoms = self.atoms.clone();

        for grant in grants {
            if let Some(idx) = self.atoms.iter().position(|a| a.atom_id == grant.atom_id) {
                // Verify grant (D11/D8/P10)
                if !grant.hash_valid() {
                    return Err(DecoherenceReject::AssistedAuthorityInvalid);
                }
                if grant.domain_id != self.domain_id || grant.policy_hash != self.policy_hash {
                    return Err(DecoherenceReject::DomainMismatch);
                }
                if grant.signature.0 == vec![0; 64] && ctx.entanglement_mode == EntanglementMode::Production {
                    return Err(DecoherenceReject::AssistedAuthorityInvalid);
                }
                
                adjusted_margins[idx] += grant.authority;
                updated_atoms[idx].cumulative_authority += grant.authority;
                // Re-finalize atom hash since metrics changed
                updated_atoms[idx].atom_hash = updated_atoms[idx].canonical_hash();
            }
        }

        if adjusted_margins.iter().all(|m| *m >= Rational64::zero()) {
            let split_witness = self.compute_grants_hash(grants);
            let cert = self.create_certificate(ctx, &adjusted_margins, cause, split_witness);
            Ok(Some(DecoherenceResult {
                state: DecoherenceState::SplitCertified,
                certificate: Some(cert),
                local_atoms: updated_atoms,
                quarantine_receipt: None,
            }))
        } else {
            Ok(None)
        }
    }

    fn quarantine(
        &self,
        _ctx: &DecoherenceContext,
        local_margins: &[Rational64],
        cause: DecoherenceCause,
    ) -> Result<DecoherenceResult, DecoherenceReject> {
        let mut failed_participants = Vec::new();
        let mut failed_margins = Vec::new();

        for (i, m) in local_margins.iter().enumerate() {
            if *m < Rational64::zero() {
                failed_participants.push(self.atoms[i].atom_id);
                failed_margins.push(*m);
            }
        }

        let mut receipt = QuarantineReceipt {
            entanglement_hash: self.entanglement_hash,
            failed_participants,
            failed_margins,
            cause,
            policy_hash: self.policy_hash,
            domain_id: self.domain_id,
            quarantine_hash: Hash32([0; 32]), // fixture_only: allow_mock
            signature: Signature(vec![0; 64]), // fixture_only: allow_mock
        };
        receipt.quarantine_hash = receipt.canonical_hash();

        Ok(DecoherenceResult {
            state: DecoherenceState::Quarantined,
            certificate: None,
            local_atoms: Vec::new(),
            quarantine_receipt: Some(receipt),
        })
    }

    fn create_certificate(
        &self,
        ctx: &DecoherenceContext,
        local_margins: &[Rational64],
        cause: DecoherenceCause,
        split_witness: Hash32,
    ) -> DecoherenceCertificate {
        let mut cert = DecoherenceCertificate {
            version: 1,
            entanglement_hash: self.entanglement_hash,
            cause,
            pre_joint_margin: self.joint_margin,
            post_local_margins: local_margins.to_vec(),
            burned_shared_defect: self.shared_defect,
            burned_shared_authority: self.shared_authority,
            redistributed_shared_defect: Rational64::zero(),
            redistributed_shared_authority: Rational64::zero(),
            participant_atom_hashes: self.atoms.iter().map(|a| a.atom_hash).collect(),
            split_witness_hash: split_witness,
            policy_hash: ctx.policy_hash,
            domain_id: ctx.domain_id,
            decoherence_hash: Hash32([0; 32]), // fixture_only: allow_mock
            signature: Signature(vec![0; 64]), // fixture_only: allow_mock
        };
        cert.decoherence_hash = cert.canonical_hash();
        cert
    }

    fn compute_hard_split_witness_hash(
        &self,
        local_margins: &[Rational64],
        cause: DecoherenceCause,
    ) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"hardsplitwitness:v1");
        hasher.update(self.domain_id.0.0);
        hasher.update(self.policy_hash.0);
        hasher.update(self.entanglement_hash.0);
        hasher.update([cause as u8]);
        for a in &self.atoms { hasher.update(a.atom_hash.0); }
        for m in local_margins {
            hasher.update(m.reduced().numer().to_be_bytes());
            hasher.update(m.reduced().denom().to_be_bytes());
        }
        Hash32(hasher.finalize().into())
    }

    fn compute_grants_hash(&self, grants: &[AuthorityGrant]) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        for g in grants {
            hasher.update(g.atom_id.0);
            hasher.update(g.authority.reduced().numer().to_be_bytes());
            hasher.update(g.authority.reduced().denom().to_be_bytes());
            hasher.update(g.receipt_hash.0);
        }
        Hash32(hasher.finalize().into())
    }
}
