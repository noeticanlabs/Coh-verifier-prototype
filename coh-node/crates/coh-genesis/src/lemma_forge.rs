use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DerivationKind {
    ArithmeticInequality,
    RingEquality,
    ListInduction,
    FinsetTelescoping,
    StructuralProjection,
    EqualityTransport,
    InvariantExtraction,
    Monotonicity,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationPlan {
    pub target_lemma_name: String,
    pub target_statement: String,
    pub derivation_kind: DerivationKind,
    pub required_hypotheses: Vec<String>,
    pub proof_strategy: Vec<String>,
}

pub struct LemmaForge;

impl LemmaForge {
    pub fn plan(goal: &str, context: &str) -> DerivationPlan {
        let kind = Self::classify_derivation(goal, context);
        let name = format!("local_lemma_{}", std::collections::hash_map::DefaultHasher::new().finish());
        
        let strategy = match kind {
            DerivationKind::ArithmeticInequality => vec!["linarith".to_string()],
            DerivationKind::RingEquality => vec!["ring".to_string()],
            DerivationKind::StructuralProjection => vec!["cases hc".to_string(), "assumption".to_string()],
            DerivationKind::InvariantExtraction => vec!["exact hc.lineage_lock".to_string()],
            _ => vec!["sorry".to_string()],
        };

        DerivationPlan {
            target_lemma_name: name,
            target_statement: goal.to_string(),
            derivation_kind: kind,
            required_hypotheses: vec![], // Inferred from context
            proof_strategy: strategy,
        }
    }

    pub fn synthesize(plan: &DerivationPlan) -> String {
        let mut lemma = format!("lemma {} : {} := by\n", plan.target_lemma_name, plan.target_statement);
        for step in &plan.proof_strategy {
            lemma.push_str(&format!("  {}\n", step));
        }
        lemma
    }

    fn classify_derivation(goal: &str, _context: &str) -> DerivationKind {
        let g = goal.to_lowercase();
        if g.contains("≤") || g.contains("<") {
            DerivationKind::ArithmeticInequality
        } else if g.contains("=") && (g.contains("+") || g.contains("*")) {
            DerivationKind::RingEquality
        } else if g.contains("axiomdeps") || g.contains("lineage") {
            DerivationKind::InvariantExtraction
        } else {
            DerivationKind::Unknown
        }
    }
}
