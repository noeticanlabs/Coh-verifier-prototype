use coh_core::atom::{CohAtom, CohGovernor, AtomMetabolism};
use coh_core::spinor::{CohSpinor, SpinContext};
use coh_core::types::{Hash32, DomainId};
use coh_core::vm::{CohVM, VerifierContext, Runtime};
use coh_genesis::vm_runtime::PhaseLoomRuntime;
use coh_npe::NpeConfig;
use num_rational::Rational64;

fn main() {
    let domain = DomainId(Hash32([1; 32]));
    let npe_config = NpeConfig::default();
    let mut runtime = PhaseLoomRuntime::new(domain, npe_config);

    let initial_state = Hash32([0; 32]);
    let misaligned_state = Hash32([0xFF; 32]);

    // High-budget Governor
    let mut governor = CohGovernor::default();
    governor.valuation = Rational64::from_integer(1000);
    governor.metabolism = AtomMetabolism {
        budget: Rational64::from_integer(100_000),
        refresh: Rational64::from_integer(100),
    };

    // Healthy initial spinor
    let mut spinor = CohSpinor::default();
    spinor.state_hash = initial_state;
    spinor.amplitude = Rational64::from_integer(1);
    spinor.norm = Rational64::from_integer(1);
    spinor.coherence_alignment = Rational64::from_integer(1);
    spinor.spinor_hash = spinor.canonical_hash();

    let spin_ctx = SpinContext {
        k1_amplification: Rational64::new(1, 10),
        k2_decay: Rational64::new(1, 100),
        tau_drift: Rational64::new(1, 1000),
    };
    let verifier_ctx = VerifierContext {
        policy_hash: Hash32([9; 32]),
        verifier_id: Hash32([10; 32]),
    };

    let mut vm = CohVM::new(initial_state, governor, spinor, spin_ctx, verifier_ctx);

    // --- SETUP: Establish a stable anchor first ---
    // Generate a valid atom to use as anchor
    for _ in 0..5 {
        vm.step(&runtime).expect("Setup step failed");
    }
    let anchor_atom = vm.finalize_atom(&mut runtime).expect("Setup finalize failed");
    
    let target_key = coh_phaseloom::PhaseKey::from_spinor(&CohSpinor::default());
    let loss_hat = Rational64::new(1, 10);
    let ctx = coh_phaseloom::CompressionContext {
        min_sources: 1, // Low min for test
        max_depth: 8,
        global_loss_hat: Rational64::new(1, 2),
        policy_hash: Hash32([0x77; 32]),
        verifier_id: Hash32([0x88; 32]),
        w_m: Rational64::from_integer(1),
        w_u: Rational64::from_integer(1),
        w_p: Rational64::from_integer(1),
    };

    let comp = runtime.loom.compress_bucket(target_key.clone(), loss_hat, &ctx).expect("Setup compression failed");
    runtime.loom.register_compression_as_anchor(comp.compression_hash).expect("Setup anchor failed");

    println!("Setup complete: Anchor installed for state_hash = initial_state");

    // --- TEST 2: Wrong final_state (Adversarial) ---
    println!("\nRunning TEST 2: Wrong final_state (Adversarial)");
    vm.state = misaligned_state; // Force VM state to mismatch anchor
    vm.spinor.state_hash = misaligned_state;
    
    let res_t2 = runtime.loom.apply_anchor_drift(&vm.spinor);
    println!("Test 2 Result: alignment={}, drift_cost={}, cone_exit={}", res_t2.alignment, res_t2.drift_cost, res_t2.cone_exit);
    assert!(res_t2.alignment < Rational64::from_integer(1));
    assert!(res_t2.cone_exit);
    assert!(res_t2.drift_cost > Rational64::from_integer(0));

    // --- TEST 3: Repeated Cone Exits -> Rephase ---
    println!("\nRunning TEST 3: Repeated Cone Exits -> Rephase");
    // rephase_trigger_count = 5 (from PhaseLoomState::new)
    for i in 0..5 {
        let res = runtime.loom.apply_anchor_drift(&vm.spinor);
        println!("Step {}: alignment={}, trigger_rephase={}, counter={}", i, res.alignment, res.trigger_rephase, runtime.loom.rephase_counter);
        if i < 4 {
            assert!(!res.trigger_rephase);
        } else {
            assert!(res.trigger_rephase);
        }
    }

    // --- TEST 4: Conflicting Policy Anchor ---
    println!("\nRunning TEST 4: Conflicting Policy Anchor");
    let conflicting_policy = Hash32([0xEE; 32]);
    runtime.loom.anchor_set.policy_anchor_hashes.push(conflicting_policy);
    // Now total_anchors increases. Even if state matches, alignment will be < 1.0 because this policy doesn't match.
    vm.spinor.state_hash = initial_state; // back to "good" state
    let res_t4 = runtime.loom.apply_anchor_drift(&vm.spinor);
    println!("Test 4 Result: alignment={}, drift_cost={}", res_t4.alignment, res_t4.drift_cost);
    assert!(res_t4.alignment < Rational64::from_integer(1));

    // --- TEST 5: High-Authority Conflict ---
    println!("\nRunning TEST 5: High-Authority Conflict");
    // We already have anchor_atom which is valid.
    runtime.loom.anchor_set.register_atom_anchor(&anchor_atom).expect("SA5 check failed");
    
    let res_t5 = runtime.loom.apply_anchor_drift(&vm.spinor);
    println!("Test 5 Result: alignment={}, drift_cost={}", res_t5.alignment, res_t5.drift_cost);

    println!("\nADVERSARIAL ANCHORING TEST PASSED");
}
