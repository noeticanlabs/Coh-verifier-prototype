use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquivalenceKind {
    DefinitionalEquality,
    PropositionalEquality,
    Isomorphism,
    GaugeEquivalence,
    Bisimulation,
    NormalFormEquivalence,
    QuotientEquivalence,
    CompressionEquivalence,
    AlphaEquivalence,
    BetaEtaEquivalence,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceDiagnosis {
    pub kind: EquivalenceKind,
    pub source_representation: String,
    pub target_representation: String,
    pub preserve_invariant: bool,
    pub proof_strategy: Option<String>,
}

pub struct EquivalenceHunter;

impl EquivalenceHunter {
    pub fn hunt(source: &str, target: &str) -> EquivalenceDiagnosis {
        let kind = Self::detect_kind(source, target);
        let strategy = match kind {
            EquivalenceKind::DefinitionalEquality => Some("rfl".to_string()),
            EquivalenceKind::NormalFormEquivalence => Some("simp; ring_nf".to_string()),
            EquivalenceKind::Isomorphism => Some("apply Isomorphism.to_equiv".to_string()),
            EquivalenceKind::GaugeEquivalence => Some("apply Gauge.equivalent".to_string()),
            EquivalenceKind::CompressionEquivalence => Some("apply Compression.bisimulation".to_string()),
            _ => None, // No fallback to sorry
        };

        EquivalenceDiagnosis {
            kind,
            source_representation: source.to_string(),
            target_representation: target.to_string(),
            preserve_invariant: true,
            proof_strategy: strategy,
        }
    }

    fn detect_kind(source: &str, target: &str) -> EquivalenceKind {
        let s = source.to_lowercase();
        let t = target.to_lowercase();

        if s == t {
            return EquivalenceKind::AlphaEquivalence;
        }

        if (s.contains("trajectory") && t.contains("atom")) || (s.contains("atom") && t.contains("trajectory")) {
            return EquivalenceKind::CompressionEquivalence;
        }

        if (s.contains("hamiltonian") && t.contains("valuation")) || (s.contains("valuation") && t.contains("hamiltonian")) {
            return EquivalenceKind::Isomorphism;
        }

        if s.contains("gauge") || t.contains("gauge") {
            return EquivalenceKind::GaugeEquivalence;
        }

        if s.contains("0") || t.contains("0") || s.contains("+") || t.contains("+") {
            return EquivalenceKind::NormalFormEquivalence;
        }

        EquivalenceKind::Unknown
    }
}
