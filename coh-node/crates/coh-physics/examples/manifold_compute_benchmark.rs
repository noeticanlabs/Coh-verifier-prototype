use coh_core::atom::{CohAtom, CohGovernor, AtomMetabolism};
use coh_core::spinor::{CohSpinor, SpinContext};
use coh_core::types::{Hash32, DomainId};
use coh_core::vm::{CohVM, VerifierContext, Runtime};
use coh_genesis::vm_runtime::PhaseLoomRuntime;
use coh_npe::NpeConfig;
use num_rational::Rational64;
use serde_json::json;
use std::time::Instant;

fn main() {
    let domain = DomainId(Hash32([1; 32]));
    let npe_config = NpeConfig::default();
    let mut runtime = PhaseLoomRuntime::new(domain, npe_config);

    let initial_state = Hash32([0; 32]);

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

    let mut report_data = vec![];

    // --- PHASE 1: COLD START ---
    let start_cold = Instant::now();
    for i in 0..50 {
        let bit = vm.step(&runtime).expect("Cold step failed");
        report_data.push(json!({
            "phase": "cold",
            "step": i,
            "bit_id": bit.bit_id.to_hex(),
            "margin": bit.margin().to_string(),
            "memory_hits": 0,
        }));
    }
    let duration_cold = start_cold.elapsed();
    let _atom_cold = vm.finalize_atom(&mut runtime).expect("Cold finalize failed");

    // --- PHASE 1.5: SYNTHETIC LOAD ---
    let start_load = Instant::now();
    for i in 0..10_000 {
        let mut synthetic_spinor = CohSpinor::default();
        synthetic_spinor.phase_num = Rational64::new(0, 1);
        synthetic_spinor.phase_den = Rational64::from_integer(1);
        synthetic_spinor.norm = Rational64::from_integer(1);
        synthetic_spinor.state_hash = initial_state;

        let mut synthetic_atom = CohAtom::identity(initial_state, Rational64::from_integer(100), domain);
        synthetic_atom.atom_id = Hash32([i as u8; 32]);
        synthetic_atom.atom_hash = synthetic_atom.canonical_hash();
        runtime.weave_to_memory(synthetic_atom, &synthetic_spinor);
    }
    let duration_load = start_load.elapsed();

    // --- PHASE 1.75: FIRST-GENERATION COMPRESSION ---
    let compress_start = Instant::now();
    let target_key = coh_phaseloom::PhaseKey::from_spinor(&CohSpinor::default());
    let loss_hat = Rational64::new(1, 10);
    let ctx = coh_phaseloom::CompressionContext {
        min_sources: 10,
        max_depth: 8,
        global_loss_hat: Rational64::new(1, 2),
        policy_hash: Hash32([0x77; 32]),
        verifier_id: Hash32([0x88; 32]),
        w_m: Rational64::from_integer(1),
        w_u: Rational64::from_integer(1),
        w_p: Rational64::from_integer(1),
    };

    let first_comp = runtime.loom.compress_bucket(target_key.clone(), loss_hat, &ctx);
    let (comp_success, first_comp_hash) = match first_comp {
        Ok(c) => (true, Some(c.compression_hash)),
        Err(_) => (false, None),
    };
    let duration_compress = compress_start.elapsed();

    // --- PHASE 1.8: RECURSIVE COMPRESSION ---
    if comp_success {
        for i in 20_000..20_050 {
            let mut synthetic_spinor = CohSpinor::default();
            synthetic_spinor.phase_num = Rational64::new(0, 1);
            synthetic_spinor.phase_den = Rational64::from_integer(1);
            synthetic_spinor.norm = Rational64::from_integer(1);
            synthetic_spinor.state_hash = initial_state;

            let mut synthetic_atom = CohAtom::identity(initial_state, Rational64::from_integer(100), domain);
            let mut id_bytes = [0xAA; 32];
            id_bytes[31] = i as u8;
            synthetic_atom.atom_id = Hash32(id_bytes);
            synthetic_atom.atom_hash = synthetic_atom.canonical_hash();
            runtime.weave_to_memory(synthetic_atom, &synthetic_spinor);
        }
        let _ = runtime.loom.compress_bucket(target_key, loss_hat, &ctx);
    }

    // --- PHASE 1.9: ANCHOR INSTALLATION (v2.3) ---
    // Register the first-generation compression as a gyroscope anchor (SA5).
    let anchor_installed = if let Some(h) = first_comp_hash {
        runtime.loom.register_compression_as_anchor(h).is_ok()
    } else {
        false
    };

    // --- PHASE 2: LONG-RUN WARM START WITH ANCHOR MONITORING (10,000 steps) ---
    vm.state = initial_state;
    vm.governor = CohGovernor::default();
    vm.governor.valuation = Rational64::from_integer(1000);
    vm.governor.metabolism = AtomMetabolism {
        budget: Rational64::from_integer(100_000),
        refresh: Rational64::from_integer(100),
    };

    let mut spinor_warm = CohSpinor::default();
    spinor_warm.state_hash = initial_state;
    spinor_warm.amplitude = Rational64::from_integer(1);
    spinor_warm.norm = Rational64::from_integer(1);
    spinor_warm.coherence_alignment = Rational64::from_integer(1);
    spinor_warm.spinor_hash = spinor_warm.canonical_hash();
    vm.spinor = spinor_warm;

    let total_warm_steps = 10_000;
    let start_warm = Instant::now();

    let mut total_drift_cost = Rational64::from_integer(0);
    let mut cone_exits = 0u32;
    let mut rephases_fired = 0u32;
    let mut min_alignment = Rational64::from_integer(1);
    let mut max_alignment = Rational64::from_integer(0);
    let mut total_alignment = Rational64::from_integer(0);

    // Sample every 200 steps to keep report compact
    let sample_interval = 200;

    for i in 0..total_warm_steps {
        let bit = vm.step(&runtime).expect("Warm step failed");
        let hits = runtime.retrieve_from_memory(&vm.spinor).len();

        // Apply anchor drift (SA1-SA3). Returns drift_cost for SA2 accounting.
        let anchor_result = runtime.loom.apply_anchor_drift(&vm.spinor);
        total_drift_cost = (total_drift_cost + anchor_result.drift_cost).reduced();

        if anchor_result.cone_exit { cone_exits += 1; }
        if anchor_result.trigger_rephase { rephases_fired += 1; }

        let a = anchor_result.alignment;
        total_alignment = (total_alignment + a).reduced();
        if a < min_alignment { min_alignment = a; }
        if a > max_alignment { max_alignment = a; }

        if i % sample_interval == 0 {
            report_data.push(json!({
                "phase": "warm_anchored",
                "step": i,
                "bit_id": bit.bit_id.to_hex(),
                "margin": bit.margin().to_string(),
                "memory_hits": hits,
                "anchor_alignment": anchor_result.alignment.to_string(),
                "drift_cost": anchor_result.drift_cost.to_string(),
                "cone_exit": anchor_result.cone_exit,
            }));
        }
    }
    let duration_warm = start_warm.elapsed();

    // --- REPORT ---
    let avg_alignment = if total_warm_steps > 0 {
        (total_alignment / Rational64::from_integer(total_warm_steps as i64)).to_string()
    } else {
        "n/a".to_string()
    };

    let final_report = json!({
        "metrics": {
            "cold_duration_ms": duration_cold.as_millis(),
            "synthetic_load_duration_ms": duration_load.as_millis(),
            "compression_duration_ms": duration_compress.as_millis(),
            "compression_success": comp_success,
            "warm_steps": total_warm_steps,
            "warm_duration_ms": duration_warm.as_millis(),

            // v2.3 Anchor metrics
            "anchor_installed": anchor_installed,
            "anchor_total_drift_cost": total_drift_cost.to_string(),
            "anchor_cone_exits": cone_exits,
            "anchor_rephases_fired": rephases_fired,
            "anchor_min_alignment": min_alignment.to_string(),
            "anchor_max_alignment": max_alignment.to_string(),
            "anchor_avg_alignment": avg_alignment,

            // Loom state
            "loom_active_threads": runtime.loom.thread_store.values().filter(|t| t.state == coh_phaseloom::ThreadState::Active).count(),
            "loom_compressed_sources": runtime.loom.thread_store.values().filter(|t| t.state == coh_phaseloom::ThreadState::CompressedSource).count(),
            "loom_compression_store_size": runtime.loom.compression_store.len(),
            "loom_max_depth": runtime.loom.metrics.max_depth,
            "loom_total_atoms": runtime.loom.metrics.total_atoms,
            "loom_rephase_counter": runtime.loom.rephase_counter,
            "loom_field_curvature": runtime.loom.metrics.field_curvature,
        },
        "sampled_steps": report_data
    });

    println!("{}", serde_json::to_string_pretty(&final_report).unwrap());
}
