use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
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
    pub suggested_lemmas: Vec<String>,
    pub proof_strategy: Vec<String>,
}

pub struct LemmaForge;

impl LemmaForge {
    pub fn plan(goal: &str, context: &str) -> DerivationPlan {
        let kind = Self::classify_derivation(goal, context);

        // Stable lemma name derived from goal content
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        goal.hash(&mut hasher);
        context.hash(&mut hasher);
        let hash_val = hasher.finish();
        let name = format!("local_lemma_{:x}", hash_val);

        let strategy = match kind {
            DerivationKind::ArithmeticInequality => vec!["linarith".to_string()],
            DerivationKind::RingEquality => vec!["ring".to_string()],
            DerivationKind::StructuralProjection => {
                vec!["cases hc".to_string(), "assumption".to_string()]
            }
            DerivationKind::InvariantExtraction => vec!["exact hc.lineage_lock".to_string()],
            _ => vec![], // No fallback to sorry; triggers escalation
        };

        // Synthesize required hypotheses and suggested lemmas from context/error
        let hypotheses = Self::synthesize_hypotheses(goal, context, kind);
        let suggested = Self::synthesize_lemmas(goal, kind);

        DerivationPlan {
            target_lemma_name: name,
            target_statement: goal.to_string(),
            derivation_kind: kind,
            required_hypotheses: hypotheses,
            suggested_lemmas: suggested,
            proof_strategy: strategy,
        }
    }

    /// Synthesize required hypotheses from goal and context
    fn synthesize_hypotheses(goal: &str, context: &str, kind: DerivationKind) -> Vec<String> {
        let mut hyps = Vec::new();
        let combined = format!("{} {}", goal, context).to_lowercase();

        // Extract typeclass requirements from context
        if combined.contains("add") || combined.contains("+") {
            hyps.push("add_comm".to_string());
        }
        if combined.contains("mul") || combined.contains("*") {
            hyps.push("mul_comm".to_string());
        }
        if combined.contains("inv") || combined.contains("inverse") {
            hyps.push("inv_exists".to_string());
        }

        // Add kind-specific hypotheses
        match kind {
            DerivationKind::ArithmeticInequality => {
                hyps.push("nonnegassumption".to_string());
            }
            DerivationKind::InvariantExtraction => {
                hyps.push("lineage_lock".to_string());
            }
            _ => {}
        }

        hyps
    }

    /// Synthesize suggested lemmas to try
    fn synthesize_lemmas(goal: &str, kind: DerivationKind) -> Vec<String> {
        let mut lemmas = Vec::new();

        match kind {
            DerivationKind::ArithmeticInequality => {
                lemmas.push("add_nonneg".to_string());
                lemmas.push("mul_nonneg".to_string());
            }
            DerivationKind::RingEquality => {
                lemmas.push("mul_distr".to_string());
                lemmas.push("add_assoc".to_string());
            }
            DerivationKind::InvariantExtraction => {
                lemmas.push("lineage_lock".to_string());
                lemmas.push("endpoint_preservation".to_string());
            }
            _ => {}
        }

        lemmas
    }

    pub fn synthesize(plan: &DerivationPlan) -> String {
        let mut lemma = format!(
            "lemma {} : {} := by\n",
            plan.target_lemma_name, plan.target_statement
        );
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
