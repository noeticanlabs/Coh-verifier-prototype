use coh_genesis::mathlib_advisor::generate_failure_report;
use coh_genesis::phaseloom_lite::{
    phaseloom_ingest, phaseloom_init, BoundaryReceiptSummary, PhaseLoomConfig,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("NPE Field Equation Production Loop v0.4 (Failure-Aware)");
    println!("========================================================");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let project_path = root_dir.join("coh-t-stack");
    let lean_file_path = project_path.join("Coh").join("Boundary").join("FieldEquation.lean");

    // 1. Initialize PhaseLoom
    let config = PhaseLoomConfig::default();
    let mut state = phaseloom_init(&config);

    // 2. Define Proof Candidates
    let candidates = vec![
        (
            "DirectTrans",
            "exact h1'.trans h2'.symm",
            "Transitivity attempt on pointwise equality"
        ),
        (
            "FixedPoint",
            "apply ENNRat.fixed_point_unique h1' h2' hl",
            "Fixed point uniqueness strategy (requires mathlib lemma)"
        ),
        (
            "Algebraic",
            "field_simp at h1' h2'; exact h1'.trans h2'.symm",
            "Field simplification strategy"
        ),
        (
            "HelperLemma",
            "have h_sub := ENNRat.sub_eq_of_add_eq h1' h2' hl; exact h_sub",
            "Subtraction-based helper lemma strategy"
        ),
    ];

    println!("Starting production sweep for 'field_equation_unique'...");

    for (name, proof, desc) in candidates {
        println!("\n[Strategy: {}] {}", name, desc);
        
        // --- Step A: Patch Lean File ---
        let original_content = fs::read_to_string(&lean_file_path).unwrap();
        // Replace only the FIRST '  sorry' (the uniqueness proof)
        let patched_content = original_content.replacen("  sorry", &format!("  {}", proof), 1);
        fs::write(&lean_file_path, patched_content).unwrap();

        // --- Step B: Run Lean Verification ---
        println!("Running lake build...");
        let output = Command::new("C:\\Users\\truea\\.elan\\bin\\lake.exe")
            .args(["build", "Coh.Boundary.FieldEquation"])
            .current_dir(&project_path)
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let full_output = format!("{}\n{}", stdout, stderr);

        // --- Step C: Generate Failure Report ---
        if let Some(report) = generate_failure_report(name, "field_equation_unique", &full_output) {
            println!("FAILURE DETECTED: Layer={:?}, Kind={:?}", report.layer, report.kind);
            println!("Normalized: {}", report.normalized_message);
            println!("Suggested Repairs: {:?}", report.suggested_repairs);

            // --- Step D: Ingest into PhaseLoom ---
            let receipt = BoundaryReceiptSummary {
                domain: "FieldEquation".to_string(),
                target: "field_equation_unique".to_string(),
                strategy_class: name.to_string(),
                accepted: false,
                novelty: 1.0,
                failure_report: Some(report),
                ..Default::default()
            };
            phaseloom_ingest(&mut state, &receipt, &config);
        } else if output.status.success() {
            println!("SUCCESS! Strategy '{}' closed the proof.", name);
            // In a real loop we would stop here
        } else {
            println!("Unclassified failure.");
        }

        // --- Step E: Restore Original Content ---
        fs::write(&lean_file_path, original_content).unwrap();
    }

    println!("\nFinal PhaseLoom Strategy Weights:");
    for (strategy, weight) in state.all_weights() {
        println!("  - {}: {:.4}", strategy, weight);
    }

    println!("\nProduction Loop Sweep Complete.");
}
