// fixture_only: allow_mock
use coh_core::atom::{CohAtom, CohGovernor, AtomMetabolism};
use coh_core::cohbit::CohBit;
use coh_core::spinor::{CohSpinor, SpinContext};
use coh_core::types::{Hash32, DomainId};
use coh_core::vm::{CohVM, VerifierContext, Runtime};
use coh_genesis::vm_runtime::PhaseLoomRuntime;
use coh_npe::NpeConfig;
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

    println!("Starting Coh-wedge Phase Loom Manifold Compute Benchmark (10,000 Steps)\n");

    // 1. Initialize VM
    let mut governor = CohGovernor::default();
    governor.valuation = Rational64::from_integer(1000000);
    governor.metabolism = AtomMetabolism {
        budget: Rational64::from_integer(100000000),
        refresh: Rational64::from_integer(1000),
    };
    let mut spinor = CohSpinor::default();
    spinor.state_hash = initial_state;
    
    let spin_ctx = SpinContext {
        k1_amplification: Rational64::new(1, 10),
        k2_decay: Rational64::new(1, 100),
        tau_drift: Rational64::new(1, 1000),
    };

    let mut vm = CohVM::new(initial_state, governor, spinor, spin_ctx, verifier_ctx);

    // 2. Cold Run (First Atom)
    println!("--- Cold Run (No memory) ---");
    let start_cold = Instant::now();
    for _ in 0..100 {
        vm.step(&runtime).expect("Cold step failed");
    }
    let atom_cold = vm.finalize_atom(&mut runtime).expect("Finalize cold atom failed");
    let duration_cold = start_cold.elapsed();
    println!("Cold atom finalized in {}ms. Hits: 0", duration_cold.as_millis());

    // 3. Weave into Phase Loom
    println!("\n--- Weaving into Phase Loom ---");
    runtime.weave_to_memory(atom_cold, &vm.spinor);

    // 4. Warm Run (Retrieval active)
    println!("\n--- Warm Run (Retrieval help) ---");
    let start_warm = Instant::now();
    for _ in 0..100 {
        vm.step(&runtime).expect("Warm step failed");
    }
    let _atom_warm = vm.finalize_atom(&mut runtime).expect("Finalize warm atom failed");
    let duration_warm = start_warm.elapsed();
    println!("Warm atom finalized in {}ms.", duration_warm.as_millis());

    // 5. 10,000 Step Stress Test (Spinor Anchoring & Locality)
    println!("\n--- 10,000 Step Stress Test (Geometric Manifold) ---");
    let start_stress = Instant::now();
    let mut rephase_count = 0;
    let mut cone_exit_count = 0;
    
    for i in 0..10000 {
        // Mock a drift update every 100 steps
        if i % 100 == 0 {
            // fixture_only: allow_mock
            let res = runtime.loom.apply_anchor_drift(&vm.spinor);
            if res.cone_exit { cone_exit_count += 1; }
            if res.trigger_rephase { rephase_count += 1; }
        }
        
        // Step VM
        vm.step(&runtime).expect("Stress step failed");
        
        // Weave small atoms periodically to flood memory
        if i % 500 == 0 {
            let mut a = CohAtom::identity(vm.state, Rational64::from_integer(100), domain);
            let mut id_bytes = [0xAA; 32]; // fixture_only: allow_mock
            id_bytes[31] = (i / 500) as u8;
            a.atom_id = Hash32(id_bytes);
            a.atom_hash = a.canonical_hash();
            runtime.weave_to_memory(a, &vm.spinor);
        }
    }
    let duration_stress = start_stress.elapsed();
    println!("10,000 steps completed in {}ms", duration_stress.as_millis());
    println!("Cone Exits: {}, Rephase Events: {}", cone_exit_count, rephase_count);
    println!("Memory Threads: {}", runtime.loom.metrics.active_threads);

    println!("\nBENCHMARK COMPLETE (Phase Loom v2.3b Stabilized)");
}
