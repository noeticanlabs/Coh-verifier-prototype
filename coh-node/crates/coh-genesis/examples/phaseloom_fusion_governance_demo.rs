use coh_genesis::{phaseloom_init, PhaseLoomConfig, verify_governed_step};
use coh_core::types_v3::{MicroReceiptV3Wire, TieredConfig, SequenceGuard, PolicyGovernance};

use coh_core::types::MicroReceipt;
use std::convert::TryFrom;

fn build_valid_wire() -> MicroReceiptV3Wire {
    let mut wire = MicroReceiptV3Wire {
        object_id: "governed_obj".to_string(),
        canon_profile_hash: "a".repeat(64),
        policy_hash: "b".repeat(64),
        state_hash_prev: "c".repeat(64),
        state_hash_next: "d".repeat(64),
        chain_digest_prev: "e".repeat(64),
        chain_digest_next: "f".repeat(64),
        step_index: 42,
        metrics: coh_core::types::MetricsWire {
            v_pre: "100".to_string(),
            v_post: "50".to_string(),
            spend: "50".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
    use coh_core::hash::compute_chain_digest;

    let v1_wire = coh_core::types::MicroReceiptWire {
        schema_id: wire.schema_id.clone(),
        version: wire.version.clone(),
        object_id: wire.object_id.clone(),
        canon_profile_hash: wire.canon_profile_hash.clone(),
        policy_hash: wire.policy_hash.clone(),
        step_index: wire.step_index,
        step_type: wire.step_type.clone(),
        signatures: wire.signatures.clone(),
        state_hash_prev: wire.state_hash_prev.clone(),
        state_hash_next: wire.state_hash_next.clone(),
        chain_digest_prev: wire.chain_digest_prev.clone(),
        chain_digest_next: wire.chain_digest_next.clone(),
        profile: coh_core::types::AdmissionProfile::CoherenceOnlyV1,
        metrics: wire.metrics.clone(),
    };

    if let Ok(r) = MicroReceipt::try_from(v1_wire) {
        let prehash = to_prehash_view(&r);
        if let Ok(canon_bytes) = to_canonical_json_bytes(&prehash) {
            let computed_digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);
            wire.chain_digest_next = computed_digest.to_hex();
        }
    }
    wire
}

fn main() {
    println!("PhaseLoom Fusion Wedge: Governed Verification Demo");
    println!("==================================================");

    let config = PhaseLoomConfig::default();
    let mut state = phaseloom_init(&config);
    state.tau = 1000;
    state.budget = 100; // Low budget

    let tiered = TieredConfig::default();
    let guard = SequenceGuard::default();
    let policy = PolicyGovernance::default();

    // 1. Create a valid-looking wire but with a projection (triggering read cost)
    let mut wire = build_valid_wire();
    wire.metrics.projection_hash = "a".repeat(64); // Simulate memory access
    wire.metrics.pl_provenance = "SIM".to_string(); // Simulate low-authority record

    // Recompute digest for PhaseLoom ecology fields
    let v1_wire = coh_core::types::MicroReceiptWire {
        schema_id: wire.schema_id.clone(),
        version: wire.version.clone(),
        object_id: wire.object_id.clone(),
        canon_profile_hash: wire.canon_profile_hash.clone(),
        policy_hash: wire.policy_hash.clone(),
        step_index: wire.step_index,
        step_type: wire.step_type.clone(),
        signatures: wire.signatures.clone(),
        state_hash_prev: wire.state_hash_prev.clone(),
        state_hash_next: wire.state_hash_next.clone(),
        chain_digest_prev: wire.chain_digest_prev.clone(),
        chain_digest_next: wire.chain_digest_next.clone(),
        profile: coh_core::types::AdmissionProfile::CoherenceOnlyV1,
        metrics: wire.metrics.clone(),
    };
    use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
    use coh_core::hash::compute_chain_digest;
    if let Ok(r) = MicroReceipt::try_from(v1_wire) {
        let prehash = to_prehash_view(&r);
        if let Ok(canon_bytes) = to_canonical_json_bytes(&prehash) {
            let computed_digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);
            wire.chain_digest_next = computed_digest.to_hex();
        }
    }

    println!("\n[SCENARIO 1] Insufficient Budget");
    state.budget = 10;
    println!("State: tau={}, budget={}", state.tau, state.budget);
    
    let (res1, _) = verify_governed_step(
        &mut state,
        &config,
        wire.clone(),
        &tiered,
        &guard,
        &policy,
        None,
        None,
    );

    println!("Decision: {:?}", res1.decision);
    println!("Message: {}", res1.message);

    // 2. Refill budget but test Anchor Firewall (simulating an attempt to mutate an EXT anchor with SIM)
    println!("\n[SCENARIO 2] Anchor Firewall Violation");
    state.budget = 10000;
    // Note: Our fusion_wedge currently compares SIM against EXT internally
    
    let (res2, _) = verify_governed_step(
        &mut state,
        &config,
        wire.clone(),
        &tiered,
        &guard,
        &policy,
        None,
        None,
    );

    println!("Decision: {:?}", res2.decision);
    println!("Message: {}", res2.message);

    println!("\n[SCENARIO 3] Successful Governed Step");
    wire.metrics.pl_provenance = "EXT".to_string(); // Use high authority
    wire.metrics.pl_tau = state.tau.to_string();
    wire.metrics.pl_budget = state.budget.to_string();
    
    // Recompute digest for Scenario 3
    let v1_wire3 = coh_core::types::MicroReceiptWire {
        schema_id: wire.schema_id.clone(),
        version: wire.version.clone(),
        object_id: wire.object_id.clone(),
        canon_profile_hash: wire.canon_profile_hash.clone(),
        policy_hash: wire.policy_hash.clone(),
        step_index: wire.step_index,
        step_type: wire.step_type.clone(),
        signatures: wire.signatures.clone(),
        state_hash_prev: wire.state_hash_prev.clone(),
        state_hash_next: wire.state_hash_next.clone(),
        chain_digest_prev: wire.chain_digest_prev.clone(),
        chain_digest_next: wire.chain_digest_next.clone(),
        profile: coh_core::types::AdmissionProfile::CoherenceOnlyV1,
        metrics: wire.metrics.clone(),
    };
    if let Ok(r) = MicroReceipt::try_from(v1_wire3) {
        let prehash = to_prehash_view(&r);
        if let Ok(canon_bytes) = to_canonical_json_bytes(&prehash) {
            let computed_digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);
            wire.chain_digest_next = computed_digest.to_hex();
        }
    }
    
    let (res3, receipt) = verify_governed_step(
        &mut state,
        &config,
        wire.clone(),
        &tiered,
        &guard,
        &policy,
        None,
        None,
    );

    println!("Decision: {:?}", res3.decision);
    println!("Message: {}", res3.message);
    if let Some(r) = receipt {
        println!("Receipt: target={}, accepted={}, budget_remaining={}", r.target, r.accepted, state.budget);
    }

    println!("\nFusion Wedge Demonstration Complete.");
}
