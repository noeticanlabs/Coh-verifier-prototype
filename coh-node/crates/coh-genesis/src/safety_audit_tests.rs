#[cfg(test)]
mod tests {
    use coh_core::rv_kernel::{RvKernel, RvGoverningState, ProtectedRvBudget};
    use coh_core::auth::{fixture_signing_key, verify_signature, VerifierContext, canonical_signed_transition_bytes};
    use coh_core::types::{MicroReceipt, Hash32, SignatureWire};
    use coh_phaseloom::{PhaseLoomState, PhaseLoomConfig, MemoryAccessPolicy, ComponentRole, MemoryView, MemoryOp, AccessDecision};
    use crate::GenesisMetrics;
    use std::hint::black_box;
    use ed25519_dalek::Signer;
    use base64::Engine as _;
    use sha2::Digest;

    #[test]
    fn test_rv_kernel_admissibility_real_path() {
        let rv = RvKernel::new(RvGoverningState::default(), ProtectedRvBudget::default());
        
        // V(x') + Spend(r) <= V(x) + Defect(r)
        // 90 + 5 <= 100 + 0  (95 <= 100) -> ACCEPT
        assert!(rv.is_admissible(black_box(90), black_box(5), black_box(100), black_box(0)));
        
        // 100 + 10 <= 100 + 0 (110 <= 100) -> REJECT
        assert!(!rv.is_admissible(black_box(100), black_box(10), black_box(100), black_box(0)));
        
        // Overflow check
        assert!(!rv.is_admissible(u128::MAX, 1, 100, 0));
    }

    #[test]
    fn test_gccp_genesis_admissibility_real_path() {
        // Law of Genesis: M(g') + C(p) <= M(g) + D(p)
        assert!(GenesisMetrics::is_genesis_admissible(100, 90, 5, 0)); // 95 <= 100
        assert!(!GenesisMetrics::is_genesis_admissible(100, 100, 10, 0)); // 110 <= 100
    }

    #[test]
    fn test_memory_view_governance_enforcement() {
        // [SECURITY ENFORCEMENT] Verifier can only read TransitionView.
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryView::TransitionView, MemoryOp::Read), AccessDecision::Allow);
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryView::AdmissionRiskView, MemoryOp::Read), AccessDecision::Deny);
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryView::ProposalContextView, MemoryOp::Read), AccessDecision::Deny);
        
        // [SECURITY ENFORCEMENT] Generator can read ProposalContextView but cannot approve.
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Generator, MemoryView::ProposalContextView, MemoryOp::Read), AccessDecision::Allow);
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Generator, MemoryView::GovernanceView, MemoryOp::Approve), AccessDecision::Deny);
    }

    #[test]
    fn test_signature_tamper_real_path() {
        let signing_key = fixture_signing_key("test_signer");
        let ctx = VerifierContext::fixture_default();
        
        // Construct a real MicroReceipt for signing
        let receipt = MicroReceipt {
            schema_id: "COH_V1".to_string(),
            version: "1".to_string(),
            object_id: "obj_0".to_string(),
            canon_profile_hash: Hash32([0; 32]),
            policy_hash: Hash32([0; 32]),
            step_index: 0,
            step_type: Some("identity".to_string()),
            signatures: None,
            state_hash_prev: Hash32([0; 32]),
            state_hash_next: Hash32([0; 32]),
            chain_digest_prev: Hash32([0; 32]),
            chain_digest_next: Hash32([0; 32]),
            profile: coh_core::types::AdmissionProfile::FormationV2,
            metrics: coh_core::types::Metrics {
                v_pre: 100,
                v_post: 90,
                spend: 10,
                defect: 0,
                m_pre: 50,
                m_post: 40,
                projection_hash: Hash32([0xAA; 32]),
                ..Default::default()
            },
        };
        
        let signed_bytes = canonical_signed_transition_bytes(&receipt, "test_signer", "*", "MICRO_RECEIPT_V1").unwrap();
        let signature = signing_key.sign(&signed_bytes);
        
        let mut sig_wire = SignatureWire {
            signature: base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
            signer: "test_signer".to_string(),
            timestamp: 1000,
            authority_id: Some("test_signer".to_string()),
            scope: Some("*".to_string()),
            expires_at: None,
        };

        // 1. Valid signature passes
        assert!(verify_signature(&receipt, &sig_wire, None, None, &ctx).is_ok());

        // 2. Tampered Object ID rejects
        let mut tampered = receipt.clone();
        tampered.object_id = "tampered_id".to_string();
        assert!(verify_signature(&tampered, &sig_wire, None, None, &ctx).is_err());

        // 3. Tampered Profile rejects (SEMANTIC BYPASS CHECK)
        let mut tampered = receipt.clone();
        tampered.profile = coh_core::types::AdmissionProfile::CoherenceOnlyV1;
        assert!(verify_signature(&tampered, &sig_wire, None, None, &ctx).is_err());

        // 4. Tampered Step Type rejects (SEMANTIC BYPASS CHECK)
        let mut tampered = receipt.clone();
        tampered.step_type = Some("malicious_step".to_string());
        assert!(verify_signature(&tampered, &sig_wire, None, None, &ctx).is_err());

        // 5. Tampered M_pre rejects (GENESIS BYPASS CHECK)
        let mut tampered = receipt.clone();
        tampered.metrics.m_pre = 999;
        assert!(verify_signature(&tampered, &sig_wire, None, None, &ctx).is_err());

        // 6. Tampered Projection Hash rejects (LINK BYPASS CHECK)
        let mut tampered = receipt.clone();
        tampered.metrics.projection_hash = Hash32([0xFF; 32]);
        assert!(verify_signature(&tampered, &sig_wire, None, None, &ctx).is_err());

        // 7. Tampered signature bytes reject
        let mut sig_corrupted = sig_wire.clone();
        sig_corrupted.signature = "A".repeat(88); // Invalid content
        assert!(verify_signature(&receipt, &sig_corrupted, None, None, &ctx).is_err());
    }

    #[derive(Default)]
    pub struct ViewGovernanceAdversarialReport {
        pub valid_passes: u64,
        pub invalid_rejections: u64,
        pub false_accepts: u64,
        pub false_rejects: u64,
        pub binding_mismatches_detected: u64,
        pub unauthorized_access_blocked: u64,
        pub stale_replays_blocked: u64,
    }

    #[test]
    fn test_view_governance_adversarial_suite() {
        let mut report = ViewGovernanceAdversarialReport::default();
        let state = PhaseLoomState::new(&PhaseLoomConfig::default());
        let receipt = coh_npe::receipt::BoundaryReceiptSummary {
            receipt_hash: hex::encode([0xAA; 32]),
            ..Default::default()
        };
        let source_hash = Hash32([0xAA; 32]);
        let policy_hash = Hash32([0; 32]); // Default in PhaseLoom
        let role = ComponentRole::Verifier;

        // 1. Golden Vector Test (Anti-self-deception)
        // [AUDIT] If the hash recipe changes, this test MUST fail.
        let golden_view = state.project_rv_view(&receipt);
        assert_eq!(golden_view.view_binding_hash.to_hex(), "1aa7f847eae4a0f366c0624ea435ac63ce5dc52c5e1e3af38be7868efdca030a", "GOLDEN VECTOR MISMATCH! Hash recipe has changed!");
        report.valid_passes += 1;

        // 2. Payload Tamper Test
        // [AUDIT] Binding must commit to payload contents.
        let mut tampered_view = golden_view.clone();
        tampered_view.spend = num_rational::Rational64::from_integer(999);
        assert!(!tampered_view.verify(&source_hash, role, &policy_hash));
        report.binding_mismatches_detected += 1;

        // 3. Schema/Version Confusion Test
        // [AUDIT] Cannot replay old layout as new layout.
        let mut schema_tampered_view = golden_view.clone();
        schema_tampered_view.schema_hash = Hash32([0xEE; 32]);
        assert!(!schema_tampered_view.verify(&source_hash, role, &policy_hash));
        report.binding_mismatches_detected += 1;

        // 4. Cross-View Replay Test
        // [AUDIT] TransitionView cannot masquerade as AdmissionRiskView.
        let gccp_view = state.project_gccp_view();
        assert_ne!(golden_view.view_binding_hash, gccp_view.view_binding_hash);
        report.binding_mismatches_detected += 1;

        // 5. Cross-Role Replay Test
        // [AUDIT] Verifier projection cannot be used by Generator.
        assert!(!golden_view.verify(&source_hash, ComponentRole::Generator, &policy_hash));
        report.binding_mismatches_detected += 1;

        // 6. Cross-Policy Replay Test
        // [AUDIT] Old policy reuse under new policy must fail.
        let new_policy = Hash32([0x01; 32]);
        assert!(!golden_view.verify(&source_hash, role, &new_policy));
        report.binding_mismatches_detected += 1;
        report.stale_replays_blocked += 1;

        // 7. Source Receipt Substitution Test
        // [AUDIT] Binding must commit to the specific source receipt.
        let source_b = Hash32([0xBB; 32]);
        assert!(!golden_view.verify(&source_b, role, &policy_hash));
        report.binding_mismatches_detected += 1;

        // 8. Unauthorized Role Access Test
        // [AUDIT] Verifier cannot request AdmissionRiskView.
        assert_eq!(MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryView::AdmissionRiskView, MemoryOp::Read), AccessDecision::Deny);
        report.unauthorized_access_blocked += 1;

        // 9. View Upgrade Attack
        // [AUDIT] TransitionView cannot be used to verify a ProposalContextView.
        let rv_view = state.project_rv_view(&receipt);
        // Simulate forgery: take rv metadata and wrap it in a GMI view
        let gmi_forgery = coh_phaseloom::ProposalContextView {
            recent_success_hashes: vec![],
            known_failure_hashes: vec![],
            approved_stable_cores: vec![],
            macro_guidance_hash: Hash32([0; 32]),
            schema_hash: rv_view.schema_hash,
            view_binding_hash: rv_view.view_binding_hash,
        };
        assert!(!gmi_forgery.verify(&source_hash, ComponentRole::Generator, &policy_hash));
        report.binding_mismatches_detected += 1;

        // 10. Randomized Mutation Fuzzer (10k)
        let mut false_accepts = 0;
        let mut rng_state: u64 = 0xDEADBEEF;
        for _ in 0..10000 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let mut view = golden_view.clone();
            
            // Randomly mutate one field, ensuring it ALWAYS changes
            match rng_state % 6 {
                0 => {
                    let new_val = num_rational::Rational64::from_integer((rng_state % 1000) as i64);
                    view.spend = if new_val == view.spend { new_val + 1 } else { new_val };
                },
                1 => {
                    let new_val = num_rational::Rational64::from_integer((rng_state % 1000) as i64);
                    view.defect = if new_val == view.defect { new_val + 1 } else { new_val };
                },
                2 => {
                    let byte = (rng_state % 255) as u8;
                    view.prev_receipt_hash = Hash32([ if byte == view.prev_receipt_hash.0[0] { byte.wrapping_add(1) } else { byte }; 32]);
                },
                3 => {
                    let byte = (rng_state % 255) as u8;
                    view.schema_hash = Hash32([ if byte == view.schema_hash.0[0] { byte.wrapping_add(1) } else { byte }; 32]);
                },
                4 => {
                    let byte = (rng_state % 255) as u8;
                    view.active_policy_hash = Hash32([ if byte == view.active_policy_hash.0[0] { byte.wrapping_add(1) } else { byte }; 32]);
                },
                _ => {
                    let byte = (rng_state % 255) as u8;
                    view.current_state_hash = Hash32([ if byte == view.current_state_hash.0[0] { byte.wrapping_add(1) } else { byte }; 32]);
                },
            }

            if view.verify(&source_hash, role, &policy_hash) {
                false_accepts += 1;
            }
        }
        report.false_accepts = false_accepts;
        assert_eq!(report.false_accepts, 0, "ADVERSARIAL FAIL: Mutation bypassed binding hash!");

        println!("\n--- View Governance Adversarial Scoreboard ---");
        println!("Valid Passes:                 {}", report.valid_passes);
        println!("Binding Mismatches Detected:  {}", report.binding_mismatches_detected);
        println!("Unauthorized Access Blocked:  {}", report.unauthorized_access_blocked);
        println!("Stale Replays Blocked:        {}", report.stale_replays_blocked);
        println!("False Accepts:                {}", report.false_accepts);
        println!("----------------------------------------------");
    }

    #[test]
    fn test_adversarial_batch_campaign_10k() {
        let rv = RvKernel::new(RvGoverningState::default(), ProtectedRvBudget::default());
        let mut false_accepts = 0;
        let mut rng_state: u64 = 0xDEADBEEF;

        for _ in 0..10000 {
            // Simple LCG for deterministic fuzzing
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let v_pre = (rng_state % 1000) as u128;
            let v_post = ((rng_state >> 8) % 1000) as u128;
            let spend = ((rng_state >> 16) % 100) as u128;
            let defect = ((rng_state >> 24) % 10) as u128;

            let is_valid = v_post + spend <= v_pre + defect;
            let rv_decision = rv.is_admissible(v_post, spend, v_pre, defect);

            if rv_decision != is_valid {
                false_accepts += 1;
            }
        }

        assert_eq!(false_accepts, 0, "RV logic diverged from law over 10k randomized samples!");
    }
}
