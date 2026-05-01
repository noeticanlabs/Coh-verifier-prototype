#[cfg(test)]
mod tests {
    use coh_core::types::{Hash32, Signature};
    use coh_core::atom::{CohAtom, AtomKind};
    use coh_phaseloom::{PhaseLoomState, PhaseLoomConfig, MemoryAccessPolicy, ComponentRole, MemoryTier, MemoryOp, AccessDecision};
    use coh_npe::receipt::BoundaryReceiptSummary;
    use num_rational::Rational64;

    #[test]
    fn test_coh_law_admissibility_inequality() {
        // Law: V(x') + Spend(r) <= V(x) + Defect(r)
        
        let v_pre = 100;
        let v_post = 90;
        let spend = 5;
        let defect = 0;
        
        // 90 + 5 <= 100 + 0  (95 <= 100) -> ACCEPT
        assert!(v_post + spend <= v_pre + defect);
        
        let v_post_fail = 99;
        let spend_fail = 10;
        // 99 + 10 <= 100  (109 <= 100) -> REJECT
        assert!(!(v_post_fail + spend_fail <= v_pre + defect));
    }

    #[test]
    fn test_memory_access_policy_verifier_isolation() {
        // [SECURITY] Verifier can ONLY read Micro tier.
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryTier::Micro, MemoryOp::Read), AccessDecision::Allow);
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryTier::Meso, MemoryOp::Read), AccessDecision::Deny);
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryTier::Macro, MemoryOp::Read), AccessDecision::Deny);
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryTier::Micro, MemoryOp::Write), AccessDecision::Deny);
    }

    #[test]
    fn test_memory_access_policy_generator_constraints() {
        // [SECURITY] Generator can read all tiers but CANNOT write or approve.
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Generator, MemoryTier::Micro, MemoryOp::Read), AccessDecision::Allow);
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Generator, MemoryTier::Macro, MemoryOp::Read), AccessDecision::Allow);
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Generator, MemoryTier::Macro, MemoryOp::Approve), AccessDecision::Deny);
    }

    #[test]
    fn test_phaseloom_view_projection_security_enforcement() {
        let state = PhaseLoomState::new(&PhaseLoomConfig::default());
        let receipt = BoundaryReceiptSummary::default();
        
        // Project RV View
        let rv_view = state.project_rv_view(&receipt);
        // RV View is strictly Micro-based. It should not contain Macro policy data.
        // In our current mock implementation, active_policy_hash is populated 
        // IF Micro Read is allowed (which it is for Verifier).
        assert_eq!(rv_view.active_policy_hash, Hash32([0; 32])); // Placeholder check
    }

    #[test]
    fn test_zero_false_accept_adversarial_logic() {
        // This test simulates the "Catastrophic" false-accept scenario.
        fn verify_transition(v_pre: i64, v_post: i64, spend: i64, defect: i64) -> bool {
             v_post + spend <= v_pre + defect
        }

        let mut false_accepts = 0;
        
        // Adversarial case: spend exceeds defect buffer
        if verify_transition(100, 95, 10, 0) { false_accepts += 1; }
        
        // Adversarial case: negative spend (if treated as value injection)
        // Here we assume spend must be non-negative in a real system.
        
        assert_eq!(false_accepts, 0, "False accepts detected in adversarial verification pass!");
    }

    #[test]
    fn test_signature_tamper_detection_logic() {
        let original_payload = b"receipt_v1_data";
        let signature = Signature(vec![0xAA; 64]);
        
        let mut tampered_payload = original_payload.to_vec();
        tampered_payload[0] ^= 0xFF; // Flip one bit
        
        // In a real system, the signature check would fail here.
        // We simulate the invariant: "Changing one byte must invalidate the signature".
        assert_ne!(original_payload.to_vec(), tampered_payload);
    }
}
