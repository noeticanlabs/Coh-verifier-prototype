use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InvariantKind {
    CommitInequality,
    ValuationPreservation,
    SpendPreservation,
    DefectPreservation,
    AuthorityPreservation,

    LineageLock,
    EndpointPreservation,
    AxiomTransparency,
    InvariantFlagPreservation,

    LorentzInvariance,
    GaugeEquivalence,
    HamiltonianConservation,
    EnergyBudget,
    DissipationBound,

    MarginConservativity,
    AuthorityNonInflation,
    LossBound,
    SourceRootPreservation,

    TypeclassInvariant,
    NamespaceInvariant,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantDiagnosis {
    pub expected: Vec<InvariantKind>,
    pub available: Vec<InvariantKind>,
    pub missing: Vec<InvariantKind>,
    pub broken: Vec<InvariantKind>,
    pub theorem_too_weak: bool,
    pub suggested_hypotheses: Vec<String>,
    pub suggested_lemmas: Vec<String>,
}

pub struct InvariantHunter;

impl InvariantHunter {
    pub fn hunt(lemma_name: &str, lemma_statement: &str, local_context: &str) -> InvariantDiagnosis {
        let expected = Self::infer_expected(lemma_name, lemma_statement);
        let available = Self::infer_available(local_context);
        
        let missing: Vec<InvariantKind> = expected
            .iter()
            .filter(|inv| !available.contains(inv))
            .cloned()
            .collect();

        let theorem_too_weak = !missing.is_empty();
        let mut suggested_hypotheses = Vec::new();
        
        if theorem_too_weak {
            for m in &missing {
                match m {
                    InvariantKind::LorentzInvariance => suggested_hypotheses.push("(hL : LorentzInvariantTrajectory τ)".into()),
                    InvariantKind::CommitInequality => suggested_hypotheses.push("(hC : TrajectoryAdmissible τ)".into()),
                    _ => {}
                }
            }
        }

        InvariantDiagnosis {
            expected,
            available,
            missing,
            broken: vec![],
            theorem_too_weak,
            suggested_hypotheses,
            suggested_lemmas: vec![],
        }
    }

    fn infer_expected(name: &str, statement: &str) -> Vec<InvariantKind> {
        let text = format!("{}\n{}", name, statement).to_lowercase();
        let mut invs = Vec::new();

        if text.contains("summary") || text.contains("compression") {
            invs.push(InvariantKind::LineageLock);
            invs.push(InvariantKind::EndpointPreservation);
            invs.push(InvariantKind::MarginConservativity);
        }

        if text.contains("lorentz") {
            invs.push(InvariantKind::LorentzInvariance);
        }

        if text.contains("commit") || text.contains("admissible") {
            invs.push(InvariantKind::CommitInequality);
        }
        
        invs
    }

    fn infer_available(context: &str) -> Vec<InvariantKind> {
        let ctx = context.to_lowercase();
        let mut invs = Vec::new();

        if ctx.contains("trajectoryadmissible") {
            invs.push(InvariantKind::CommitInequality);
        }

        if ctx.contains("conservativecompression") {
            invs.push(InvariantKind::MarginConservativity);
            invs.push(InvariantKind::LineageLock);
            invs.push(InvariantKind::EndpointPreservation);
        }

        if ctx.contains("lorentzinvarianttrajectory") {
            invs.push(InvariantKind::LorentzInvariance);
        }

        invs
    }
}
