// fixture_only: allow_mock
use coh_core::atom::{CohGovernor, AtomMetabolism};
use coh_core::spinor::{CohSpinor, SpinContext};
use coh_core::types::{Hash32, DomainId};
use coh_core::vm::{CohVM, VerifierContext, Runtime};
use coh_core::ignition::{IgnitionTracker, IgnitionThresholds, classify_ignition, IgnitionState};
use coh_core::stabilization::{StabilizationTracker, StabilizationThresholds, classify_stability};
use coh_genesis::vm_runtime::PhaseLoomRuntime;
use coh_npe::NpeConfig;
use coh_phaseloom::{CompressionContext, PhaseKey};
use num_rational::Rational64;
use num_traits::One;
use std::time::Instant;

fn main() {
    let domain = DomainId(Hash32([1; 32]));
    let npe_config = NpeConfig::default();
    let mut runtime = PhaseLoomRuntime::new(domain, npe_config);
    let initial_state = Hash32([0; 32]);
    let verifier_ctx = VerifierContext {
        policy_hash: Hash32([9; 32]),
        verifier_id: Hash32([10; 32]),
    };

    println!("Starting Coh-wedge Complex Trajectory Production Stress Test\n");

    // 1. Setup Base VM
    let mut governor = CohGovernor::default();
    governor.valuation = Rational64::from_integer(1_000_000);
    governor.metabolism = AtomMetabolism {
        budget: Rational64::from_integer(100_000_000),
        refresh: Rational64::from_integer(1000),
    };
    let mut spinor = CohSpinor::default();
    spinor.domain = domain;
    let spin_ctx = SpinContext {
        k1_amplification: Rational64::new(1, 10),
        k2_decay: Rational64::new(1, 100),
        tau_drift: Rational64::new(1, 1000),
    };

    let mut master_vm = CohVM::new(initial_state, governor, spinor, spin_ctx, verifier_ctx);
    let mut i_tracker = IgnitionTracker::new(20); // Smaller window for faster state transitions
    let mut s_tracker = StabilizationTracker::new(20);

    let start_time = Instant::now();

    // --- PHASE 1: BRANCHING PRODUCTION ---
    println!("--- PHASE 1: BRANCHING PRODUCTION (Generation of 20 Atoms) ---");
    for gen in 0..20 {
        let mut branches: Vec<CohVM> = (0..5).map(|_| master_vm.clone()).collect();
        let mut best_idx = 0;
        let mut best_score = Rational64::from_integer(-1);

        for (i, branch) in branches.iter_mut().enumerate() {
            for _ in 0..10 {
                let r: &PhaseLoomRuntime = &runtime;
                branch.step(r).expect("Branch step failed");
            }
            let alignment = runtime.loom.anchor_set.compute_alignment(
                &branch.spinor, 
                &runtime.loom.thread_store, 
                &runtime.loom.compression_store
            );
            let score = branch.governor.valuation * alignment;
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        master_vm = branches[best_idx].clone();
        let anchor_spinor = master_vm.spinor.clone();
        let atom = master_vm.finalize_atom(&mut runtime).expect("Finalize failed");
        runtime.weave_to_memory(atom, &anchor_spinor);
        
        i_tracker.record_step(master_vm.governor.valuation, Rational64::new(98, 100), Rational64::new(80, 100), Rational64::new(95, 100), Rational64::one(), Rational64::from_integer(10));
        s_tracker.record_step(master_vm.governor.valuation, Rational64::new(5, 10), Rational64::new(98, 100), Rational64::from_integer(10));

        if gen % 5 == 0 { println!("  Generation {}: Winner Score = {}", gen, best_score); }
    }

    // --- PHASE 2: THE QUENCH ---
    println!("\n--- PHASE 2: THE QUENCH (Margin Collapse Injection) ---");
    master_vm.governor.valuation = Rational64::from_integer(5); // Near-zero margin
    for _ in 0..25 { // Flush window (20)
        i_tracker.record_step(master_vm.governor.valuation, Rational64::new(30, 100), Rational64::new(10, 100), Rational64::new(20, 100), Rational64::from_integer(50), Rational64::from_integer(100));
        s_tracker.record_step(master_vm.governor.valuation, Rational64::new(1, 10), Rational64::new(30, 100), Rational64::from_integer(100));
    }
    let m_quench = i_tracker.compute_metrics();
    println!("State after Quench: {:?}", classify_ignition(&m_quench, &IgnitionThresholds::default()));
    assert_eq!(classify_ignition(&m_quench, &IgnitionThresholds::default()), IgnitionState::Cold);

    // --- PHASE 3: RE-IGNITION VIA MEMORY ---
    println!("\n--- PHASE 3: RE-IGNITION (Anchored Recovery) ---");
    let anchor_spinor = master_vm.spinor.clone();
    let target_key = PhaseKey::from_spinor(&anchor_spinor);
    let comp_ctx = CompressionContext {
        min_sources: 2, max_depth: 8, global_loss_hat: Rational64::new(1, 2),
        policy_hash: Hash32([0x77; 32]), verifier_id: Hash32([0x88; 32]),
        w_m: Rational64::one(), w_u: Rational64::one(), w_p: Rational64::one(),
    };
    if let Ok(comp_res) = runtime.loom.compress_bucket(target_key, Rational64::new(1, 10), &comp_ctx) {
        runtime.loom.register_compression_as_anchor(comp_res.compression_hash).unwrap();
        println!("  Anchor registered. Recovering...");
    }

    for _ in 0..100 {
        master_vm.governor.valuation += Rational64::from_integer(10000); // Injection
        i_tracker.record_step(master_vm.governor.valuation, Rational64::new(99, 100), Rational64::new(95, 100), Rational64::new(99, 100), Rational64::one(), Rational64::from_integer(5));
        s_tracker.record_step(master_vm.governor.valuation, Rational64::new(5, 10), Rational64::new(99, 100), Rational64::from_integer(5));
    }
    
    let m_final = i_tracker.compute_metrics();
    let s_final = s_tracker.compute_metrics();
    println!("Final Status: {:?} | {:?}", classify_ignition(&m_final, &IgnitionThresholds::default()), classify_stability(&s_final, &StabilizationThresholds::default()));
    
    println!("\nCOMPLEX PRODUCTION TEST COMPLETE. Time: {:?}", start_time.elapsed());
}
