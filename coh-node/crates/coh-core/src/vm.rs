use crate::cohbit::{CohBit, CohBitReject};
use crate::atom::{CohAtom, CohGovernor, CohAtomReject};
use crate::spinor::{CohSpinor, SpinContext};
use crate::types::Hash32;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use num_rational::Rational64;

#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum VmReject {
    #[error("No admissible transition from state {0}")]
    NoAdmissibleTransition(String),
    #[error("Execution mismatch: expected {expected}, got {actual}")]
    ExecutionMismatch { expected: String, actual: String },
    #[error("Bit rejected by kernel: {0}")]
    BitRejected(CohBitReject),
    #[error("Atom rejection: {0}")]
    AtomRejected(CohAtomReject),
    #[error("Budget exhausted")]
    BudgetExhausted,
    #[error("Policy violation")]
    PolicyViolation,
}

/// Runtime Trait: The interface for concrete state mutation and memory manifold interaction.
pub trait Runtime {
    fn execute(&self, action: Hash32) -> Hash32;
    fn propose_candidates(&self, state: Hash32) -> Vec<CohBit>;
    
    /// Phase matching retrieval from the Loom.
    fn retrieve_from_memory(&self, spinor: &CohSpinor) -> Vec<CohAtom>;
    
    /// Weaving a verified trajectory into the Loom.
    fn weave_to_memory(&mut self, atom: CohAtom, final_spinor: &CohSpinor);
}

/// Verifier Context: The policy and verifier authority.
pub struct VerifierContext {
    pub policy_hash: Hash32,
    pub verifier_id: Hash32,
}

/// CohVM: The verifier-gated metabolic state machine with Loom integration.
pub struct CohVM {
    pub state: Hash32,
    pub governor: CohGovernor,
    pub spinor: CohSpinor,
    pub spin_ctx: SpinContext,
    pub verifier_ctx: VerifierContext,
    
    pub current_atom: Option<CohAtom>,
    pub initial_valuation: Rational64,
}

impl CohVM {
    pub fn new(
        initial_state: Hash32,
        governor: CohGovernor,
        spinor: CohSpinor,
        spin_ctx: SpinContext,
        verifier_ctx: VerifierContext,
    ) -> Self {
        let initial_valuation = governor.valuation;
        Self {
            state: initial_state,
            governor,
            spinor,
            spin_ctx,
            verifier_ctx,
            current_atom: None,
            initial_valuation,
        }
    }

    /// The Runtime Loop: Propose -> Filter -> Retrieve -> Route -> Execute -> Verify -> Update
    pub fn step(&mut self, runtime: &dyn Runtime) -> Result<CohBit, VmReject> {
        // 1. Propose candidates (Exploration)
        let mut candidates = runtime.propose_candidates(self.state);

        // 2. Memory Enrichment (The Weave)
        let memory_hits = runtime.retrieve_from_memory(&self.spinor);
        for atom in memory_hits {
            for bit in &atom.bits {
                if bit.from_state == self.state {
                    candidates.push(bit.clone());
                }
            }
        }

        // 3. Filter by Admissibility (K2, K4, K5)
        let valid: Vec<CohBit> = candidates
            .into_iter()
            .filter(|b| b.executable() && b.from_state == self.state)
            .collect();

        if valid.is_empty() {
            return Err(VmReject::NoAdmissibleTransition(self.state.to_hex()));
        }

        // 4. Route by Spinor Preference (K7)
        let weighted = self.spinor.route_candidates(&valid)
            .map_err(|_| VmReject::PolicyViolation)?;
        
        let mut chosen = weighted.first()
            .ok_or(VmReject::NoAdmissibleTransition(self.state.to_hex()))?.0.clone();

        // 4b. Link into chain (Continuity Laws)
        if let Some(ref atom) = self.current_atom {
            if let Some(prev) = atom.bits.last() {
                chosen.step_index = prev.step_index + 1;
                chosen.prev_receipt_hash = Some(prev.receipt_hash);
                chosen.chain_digest_pre = prev.chain_digest_post;
            }
        }
        chosen.chain_digest_post = chosen.chain_digest_pre.combine_tagged("cohbit:v1:chain", &chosen.action_hash); // Mock derivation
        chosen.receipt_hash = chosen.canonical_hash();

        // 5. Execute and Verify (K6)
        let next_state = runtime.execute(chosen.action_hash);
        if next_state != chosen.to_state {
            return Err(VmReject::ExecutionMismatch {
                expected: chosen.to_state.to_hex(),
                actual: next_state.to_hex(),
            });
        }

        // 6. Update Governor (Metabolic Law)
        if !self.governor.evolve(&chosen) {
            return Err(VmReject::BudgetExhausted);
        }

        // 7. Evolve Spinor (Adaptive Control)
        self.spinor = self.spinor.evolve(&chosen, &self.spin_ctx)
            .map_err(|_| VmReject::PolicyViolation)?;

        // 8. Track in current atom
        if let Some(ref mut atom) = self.current_atom {
            atom.bits.push(chosen.clone());
            atom.cumulative_spend += chosen.spend;
            atom.cumulative_defect += chosen.defect;
            atom.cumulative_delta_hat += chosen.delta_hat;
            atom.cumulative_authority += chosen.authority;
        } else {
            self.current_atom = Some(CohAtom {
                initial_state: self.state,
                bits: vec![chosen.clone()],
                cumulative_spend: chosen.spend,
                cumulative_defect: chosen.defect,
                cumulative_delta_hat: chosen.delta_hat,
                cumulative_authority: chosen.authority,
                domain: chosen.domain,
                policy_hash: self.verifier_ctx.policy_hash,
                verifier_id: self.verifier_ctx.verifier_id,
                ..Default::default()
            });
        }

        self.state = next_state;
        Ok(chosen)
    }

    /// Finalize and close the current atom block (K8)
    pub fn finalize_atom(&mut self, runtime: &mut dyn Runtime) -> Result<CohAtom, VmReject> {
        let mut atom = self.current_atom.take().ok_or(VmReject::NoAdmissibleTransition("empty atom".to_string()))?;
        atom.final_state = self.state;
        
        // Compute total margin (A22)
        atom.margin_total = self.initial_valuation + atom.cumulative_defect + atom.cumulative_authority - self.governor.valuation - atom.cumulative_spend;

        // Set canonical hash before verification
        atom.atom_hash = atom.canonical_hash();

        // Final verification of atom laws (A0-A22)
        if !atom.hash_valid() {
            return Err(VmReject::AtomRejected(CohAtomReject::BadAtomHash));
        }
        if let Err(e) = atom.continuity_valid() {
            return Err(VmReject::AtomRejected(e));
        }
        if let Err(e) = atom.metrics_valid() {
            return Err(VmReject::AtomRejected(e));
        }
        if let Err(e) = atom.budget_valid(self.initial_valuation, self.governor.valuation) {
            return Err(VmReject::AtomRejected(e));
        }
        
        // Weave the trajectory into memory
        runtime.weave_to_memory(atom.clone(), &self.spinor);
        
        // Prepare for next atom
        self.initial_valuation = self.governor.valuation;
        
        Ok(atom)
    }
}
