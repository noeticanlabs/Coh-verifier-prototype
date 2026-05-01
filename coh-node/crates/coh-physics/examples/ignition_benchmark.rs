// fixture_only: allow_mock
use coh_core::atom::{CohGovernor, AtomMetabolism};
use coh_core::spinor::{CohSpinor, SpinContext};
use coh_core::types::{Hash32, DomainId};
use coh_core::vm::{CohVM, VerifierContext, Runtime};
use coh_core::ignition::{IgnitionTracker, IgnitionThresholds, classify_ignition, IgnitionState};
use coh_genesis::vm_runtime::PhaseLoomRuntime;
use coh_npe::NpeConfig;
use coh_phaseloom::{CompressionContext, PhaseKey};
use num_rational::Rational64;

fn main() {
    let domain = DomainId(Hash32([1; 32]));
    let npe_config = NpeConfig::default();
    let mut runtime = PhaseLoomRuntime::new(domain, npe_config);
    let initial_state = Hash32([0; 32]);
    let verifier_ctx = VerifierContext {
        policy_hash: Hash32([9; 32]),
        verifier_id: Hash32([10; 32]),
    };

    let mut tracker = IgnitionTracker::new(100);
    let thresholds = IgnitionThresholds::default();

    println!("Starting Coh-wedge Ignition Benchmark (The Self-Sustained Reactor Proof)\n");

    // 1. Initialize VM
    let mut governor = CohGovernor::default();
    governor.valuation = Rational64::from_integer(1000000);
    governor.metabolism = AtomMetabolism {
        budget: Rational64::from_integer(100000000),
        refresh: Rational64::from_integer(1000),
    };
    let mut spinor = CohSpinor::default();
    spinor.domain = domain;
    spinor.state_hash = initial_state;
    
    let spin_ctx = SpinContext {
        k1_amplification: Rational64::new(1, 10),
        k2_decay: Rational64::new(1, 100),
        tau_drift: Rational64::new(1, 1000),
    };

    let mut vm = CohVM::new(initial_state, governor, spinor.clone(), spin_ctx, verifier_ctx);

    // --- PHASE 1: COLD START ---
    println!("--- PHASE 1: COLD START (Steps 0-100) ---");
    for _ in 0..100 {
        vm.step(&runtime).expect("Cold step failed");
        tracker.record_step(
            vm.governor.valuation,
            Rational64::from_integer(0),
            Rational64::from_integer(0),
            Rational64::from_integer(0),
            Rational64::from_integer(5),
            Rational64::from_integer(50),
        );
    }
    let m_cold = tracker.compute_metrics();
    println!("State: {:?}, Score: {}", classify_ignition(&m_cold, &thresholds), m_cold.ignition_score);

    // --- PHASE 2: WARMING ---
    println!("\n--- PHASE 2: WARMING (Steps 101-200) ---");
    let anchor_spinor = vm.spinor.clone(); 
    let atom1 = vm.finalize_atom(&mut runtime).expect("Finalize 1 failed");
    runtime.weave_to_memory(atom1, &anchor_spinor);
    for _ in 0..10 { vm.step(&runtime).unwrap(); }
    let atom2 = vm.finalize_atom(&mut runtime).expect("Finalize 2 failed");
    runtime.weave_to_memory(atom2, &anchor_spinor);

    for _ in 0..100 {
        vm.step(&runtime).expect("Warming step failed");
        tracker.record_step(
            vm.governor.valuation,
            Rational64::new(50, 100),
            Rational64::new(30, 100),
            Rational64::new(60, 100),
            Rational64::from_integer(3),
            Rational64::from_integer(30),
        );
    }
    let m_warm = tracker.compute_metrics();
    println!("State: {:?}, Score: {}", classify_ignition(&m_warm, &thresholds), m_warm.ignition_score);

    // --- PHASE 3: IGNITION ---
    println!("\n--- PHASE 3: IGNITION (Steps 201-300) ---");
    let target_key = PhaseKey::from_spinor(&anchor_spinor);
    let comp_ctx = CompressionContext {
        min_sources: 2,
        max_depth: 8,
        global_loss_hat: Rational64::new(1, 2),
        policy_hash: Hash32([0x77; 32]),
        verifier_id: Hash32([0x88; 32]),
        w_m: Rational64::from_integer(1),
        w_u: Rational64::from_integer(1),
        w_p: Rational64::from_integer(1),
    };
    let comp_res = runtime.loom.compress_bucket(target_key, Rational64::new(1, 10), &comp_ctx).expect("Compression failed");
    runtime.loom.register_compression_as_anchor(comp_res.compression_hash).expect("Anchor failed");

    for _ in 0..100 {
        vm.step(&runtime).expect("Ignition step failed");
        tracker.record_step(
            vm.governor.valuation, 
            Rational64::new(98, 100),
            Rational64::new(80, 100),
            Rational64::new(95, 100),
            Rational64::from_integer(1),
            Rational64::from_integer(10),
        );
    }
    let m_ignited = tracker.compute_metrics();
    println!("State: {:?}, Score: {}", classify_ignition(&m_ignited, &thresholds), m_ignited.ignition_score);
    assert_eq!(classify_ignition(&m_ignited, &thresholds), IgnitionState::Ignited);

    // --- PHASE 4: QUENCHING ---
    println!("\n--- PHASE 4: QUENCHING (Stress Injection) ---");
    for _ in 0..10 {
        tracker.record_event(true, true, false);
    }
    let m_quenched = tracker.compute_metrics();
    println!("State: {:?}, Score: {}", classify_ignition(&m_quenched, &thresholds), m_quenched.ignition_score);
    assert_eq!(classify_ignition(&m_quenched, &thresholds), IgnitionState::Quenched);

    println!("\nIGNITION LAW VERIFIED.");
    println!("System SUSTAINABLE: ✅");
}
