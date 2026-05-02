#[cfg(test)]
mod tests {
    use coh_core::types::Hash32;

    #[derive(Debug, PartialEq, Eq)]
    enum VerifyResult {
        Accept,
        Reject(String),
    }

    struct SummaryAtom {
        source_root: String,
        axiom_dependencies: Vec<String>,
        invariant_flags: Vec<String>,
        authority_upper_bound: u128,
        defect_upper_bound: u128,
        initial_state_hash: String,
        final_state_hash: String,
    }

    fn verify_summary(atom: &SummaryAtom, source_axioms: &[String]) -> VerifyResult {
        // 1. Lineage check
        if atom.source_root.is_empty() {
            return VerifyResult::Reject("lineage_mismatch".to_string());
        }

        // 2. Axiom inheritance check (Exact match)
        if atom.axiom_dependencies.len() != source_axioms.len() ||
           !atom.axiom_dependencies.iter().all(|a| source_axioms.contains(a)) {
            return VerifyResult::Reject("axiom_smuggling".to_string());
        }

        // 3. Invariant check
        if !atom.invariant_flags.contains(&"LorentzInvariant".to_string()) {
            return VerifyResult::Reject("missing_lorentz_flag".to_string());
        }

        // 4. Budget checks (Simplified)
        if atom.authority_upper_bound > 100 {
            return VerifyResult::Reject("authority_inflation".to_string());
        }
        if atom.defect_upper_bound > 50 {
            return VerifyResult::Reject("margin_inflation".to_string());
        }

        VerifyResult::Accept
    }

    #[test]
    fn test_rejects_lineage_mismatch() {
        let mut atom = valid_atom();
        atom.source_root = String::new();
        assert_eq!(verify_summary(&atom, &source_axioms()), VerifyResult::Reject("lineage_mismatch".to_string()));
    }

    #[test]
    fn test_rejects_axiom_smuggling() {
        let mut atom = valid_atom();
        atom.axiom_dependencies = vec![]; // Dropped axiom
        assert_eq!(verify_summary(&atom, &source_axioms()), VerifyResult::Reject("axiom_smuggling".to_string()));
    }

    #[test]
    fn test_rejects_missing_lorentz_flag() {
        let mut atom = valid_atom();
        atom.invariant_flags = vec![]; 
        assert_eq!(verify_summary(&atom, &source_axioms()), VerifyResult::Reject("missing_lorentz_flag".to_string()));
    }

    #[test]
    fn test_rejects_authority_inflation() {
        let mut atom = valid_atom();
        atom.authority_upper_bound = 1000;
        assert_eq!(verify_summary(&atom, &source_axioms()), VerifyResult::Reject("authority_inflation".to_string()));
    }

    #[test]
    fn test_rejects_margin_inflation() {
        let mut atom = valid_atom();
        atom.defect_upper_bound = 500;
        assert_eq!(verify_summary(&atom, &source_axioms()), VerifyResult::Reject("margin_inflation".to_string()));
    }

    fn source_axioms() -> Vec<String> {
        vec!["current_conservation".to_string()]
    }

    fn valid_atom() -> SummaryAtom {
        SummaryAtom {
            source_root: "merkle_root_123".to_string(),
            axiom_dependencies: vec!["current_conservation".to_string()],
            invariant_flags: vec!["LorentzInvariant".to_string()],
            authority_upper_bound: 100,
            defect_upper_bound: 50,
            initial_state_hash: "0x1".to_string(),
            final_state_hash: "0x2".to_string(),
        }
    }
}
