// fixture_only: allow_mock
use coh_core::atom::{CohGovernor, AtomMetabolism};
use coh_core::spinor::{CohSpinor, SpinContext};
use coh_core::types::{Hash32, DomainId};
use coh_core::vm::{CohVM, VerifierContext, Runtime};
use coh_core::ignition::{IgnitionTracker, IgnitionThresholds};
use coh_core::stabilization::{StabilizationTracker, StabilizationThresholds};
use coh_genesis::vm_runtime::PhaseLoomRuntime;
use coh_npe::NpeConfig;
use coh_phaseloom::{CompressionContext, PhaseKey};
use num_rational::Rational64;
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

    println!("Starting Coh-wedge Stress and Possibility Suite\n");

    // 1. PERFORMANCE TEST: HIGH-THROUGHPUT EXECUTION
    println!("--- TEST 1: HIGH-THROUGHPUT EXECUTION ---");
    let mut governor = CohGovernor::default();
    governor.valuation = Rational64::from_integer(1_000_000_000);
    governor.metabolism = AtomMetabolism {
        budget: Rational64::from_integer(100_000_000_000),
        refresh: Rational64::from_integer(100_000),
    };
    let mut spinor = CohSpinor::default();
    spinor.domain = domain;
    
    let spin_ctx = SpinContext {
        k1_amplification: Rational64::new(1, 10),
        k2_decay: Rational64::new(1, 100),
        tau_drift: Rational64::new(1, 1000),
    };

    let mut vm = CohVM::new(initial_state, governor, spinor.clone(), spin_ctx, verifier_ctx);
    
    let start = Instant::now();
    let iterations = 10000;
    for _ in 0..iterations {
        vm.step(&runtime).expect("Step failed under load");
    }
    let duration = start.elapsed();
    println!("Processed {} bits in {:?}. ({:.2} bits/sec)", 
        iterations, duration, iterations as f64 / duration.as_secs_f64());

    // 2. POSSIBILITY TEST: DEEP FRACTAL COMPRESSION
    println!("\n--- TEST 2: DEEP FRACTAL COMPRESSION ---");
    let anchor_spinor = vm.spinor.clone();
    let target_key = PhaseKey::from_spinor(&anchor_spinor);
    
    // Weave many atoms to fill the bucket
    for i in 0..20 {
        for _ in 0..10 { vm.step(&runtime).unwrap(); }
        let atom = vm.finalize_atom(&mut runtime).expect("Finalize failed");
        runtime.weave_to_memory(atom, &anchor_spinor);
        if i % 5 == 0 { println!("  Weaved atom {}...", i); }
    }

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

    let comp_start = Instant::now();
    let comp_res = runtime.loom.compress_bucket(target_key.clone(), Rational64::new(1, 10), &comp_ctx).expect("Compression failed");
    println!("Compressed bucket in {:?}. Summary Atom Hash: {:?}", comp_start.elapsed(), comp_res.compression_hash);
    println!("Compression depth: {}, Source count: {}", comp_res.lineage.depth, comp_res.source_count);

    // 3. DYNAMICAL TEST: IGNITION + STABILIZATION UNDER LOAD
    println!("\n--- TEST 3: IGNITION + STABILIZATION UNDER LOAD ---");
    let mut i_tracker = IgnitionTracker::new(100);
    let mut s_tracker = StabilizationTracker::new(100);
    let i_thresholds = IgnitionThresholds::default();
    let s_thresholds = StabilizationThresholds::default();

    for _ in 0..200 {
        vm.step(&runtime).expect("Step failed");
        i_tracker.record_step(
            vm.governor.valuation,
            Rational64::new(99, 100),
            Rational64::new(90, 100),
            Rational64::new(98, 100),
            Rational64::from_integer(1),
            Rational64::from_integer(10),
        );
        s_tracker.record_step(
            vm.governor.valuation,
            Rational64::new(5, 10),
            Rational64::new(99, 100),
            Rational64::from_integer(10),
        );
    }

    let i_metrics = i_tracker.compute_metrics();
    let s_metrics = s_tracker.compute_metrics();
    
    println!("Final Reactor Status:");
    println!("  Ignition Score: {}", i_metrics.ignition_score);
    println!("  Stability Score: {}", s_metrics.stability_score);
    println!("  Combined State: {:?} | {:?}", 
        coh_core::ignition::classify_ignition(&i_metrics, &i_thresholds),
        coh_core::stabilization::classify_stability(&s_metrics, &s_thresholds));

    println!("\nSTRESS SUITE COMPLETE.");
}
