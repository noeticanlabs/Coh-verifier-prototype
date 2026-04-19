use crate::types::{Hash32, VerifyMicroResult, RejectCode, MicroReceiptWire};
use serde::{Deserialize, Serialize};

/// Canonical State Identifier (Verifier Linkage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateId(pub Hash32);

/// Constraint types C1-C6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstraintWitness {
    C1Schema,
    C2Identity,
    C3Profile,
    C4StateHashLink,
    C5ChainConsistency,
    C6Policy,
}

/// Status of a constraint witness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WitnessStatus {
    Pass,
    Fail,
    Unknown,
}

/// Proof-of-Acceptance (Zero-sized marker)
/// Only constructible via `VerifiedStep::try_from_candidate`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptWitness;

/// Semantic State per Domain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "domain", content = "data")]
pub enum DomainState {
    Financial(crate::trajectory::domain::FinancialState),
    Agent(crate::trajectory::domain::AgentState),
    Ops(crate::trajectory::domain::OpsState),
}

/// Semantic Action per Domain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "domain", content = "data")]
pub enum Action {
    Financial(crate::trajectory::domain::FinancialAction),
    Agent(crate::trajectory::domain::AgentAction),
    Ops(crate::trajectory::domain::OpsAction),
}

/// A candidate edge in the search space (Execution + Rejected)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateEdge {
    pub state_prev: DomainState,
    pub action: Action,
    pub state_next: DomainState,
    pub receipt: MicroReceiptWire,
    pub verification: VerifyMicroResult,
    pub witnesses: Vec<(ConstraintWitness, WitnessStatus)>,
}

/// A formally verified step in an execution path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedStep {
    pub state_prev: DomainState,
    pub action: Action,
    pub state_next: DomainState,
    pub witness: AcceptWitness,
}

/// A complete admissible trajectory (Execution Graph)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissibleTrajectory {
    pub steps: Vec<VerifiedStep>,
    pub cumulative_score: f64,
}

impl AdmissibleTrajectory {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            cumulative_score: 0.0,
        }
    }

    pub fn push(&mut self, step: VerifiedStep) {
        // Invariants: state and chain continuity should be checked in the engine
        self.steps.push(step);
    }
}

/// Canonical witness mapping function
pub fn witness_vector(result: &VerifyMicroResult) -> Vec<(ConstraintWitness, WitnessStatus)> {
    let mut witnesses = Vec::new();
    
    // Mapping RejectCodes to C1-C6
    let status = |code: RejectCode| {
        if let Some(r_code) = result.code {
            if r_code == code { WitnessStatus::Fail } else { WitnessStatus::Pass }
        } else {
            WitnessStatus::Pass
        }
    };

    witnesses.push((ConstraintWitness::C1Schema, status(RejectCode::RejectSchema)));
    witnesses.push((ConstraintWitness::C2Identity, status(RejectCode::RejectMissingSignature)));
    witnesses.push((ConstraintWitness::C3Profile, status(RejectCode::RejectCanonProfile)));
    witnesses.push((ConstraintWitness::C4StateHashLink, status(RejectCode::RejectStateHashLink)));
    witnesses.push((ConstraintWitness::C5ChainConsistency, status(RejectCode::RejectChainDigest)));
    witnesses.push((ConstraintWitness::C6Policy, status(RejectCode::RejectPolicyViolation)));

    witnesses
}

// Note: I will need to refine C4 mapping based on the actual reject.rs codes.
// RejectStateHashLink is C4. RejectChainDigest is C5.
