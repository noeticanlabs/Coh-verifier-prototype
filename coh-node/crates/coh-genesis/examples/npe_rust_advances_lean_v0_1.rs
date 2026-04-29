use coh_genesis::failure_taxonomy::{FailureLayer, FailureSeverity};
use coh_genesis::mathlib_advisor::{generate_failure_report, MathlibLakeQuery};
use coh_genesis::phaseloom_lite::{
    phaseloom_ingest, phaseloom_init, BoundaryReceiptSummary, PhaseLoomConfig,
    MathlibEffect,
};
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("NPE Rust Advances NPE Lean Loop v0.1");
    println!("======================================");

    let project_path = PathBuf::from(r"c:\Users\truea\OneDrive\Desktop\Coh-wedge\coh-t-stack");
    let target_file = project_path
        .join("Coh")
        .join("Boundary")
        .join("FieldEquation.lean");

    // 1. Initialize PhaseLoom
    let config = PhaseLoomConfig::default();
    let mut phaseloom = phaseloom_init(&config);
    let mut advisor = MathlibLakeQuery::new(project_path.clone());

    // 2. Identify the stuck theorem and the missing pattern
    let target_theorem = "field_equation_unique";
    let missing_pattern = "sub_eq_of_add_eq"; // Derived from previous failures

    println!("Target: {}", target_theorem);
    println!("Detected Missing Pattern: {}", missing_pattern);

    // 3. [NPE-Rust] Synthesize the missing lemma
    println!("\n[NPE-Rust] Synthesizing missing lemma...");
    if let Some((lemma_name, lemma_body)) =
        advisor.synthesizer.synthesize("ENNRat", missing_pattern)
    {
        println!("SUCCESS: Synthesized '{}'", lemma_name);
        println!("Body: {}", lemma_body);

        // 4. [NPE-Rust] Patch the Lean file with the new lemma
        println!("\n[NPE-Rust] Patching source with synthesized lemma...");
        let content = std::fs::read_to_string(&target_file).expect("Failed to read Lean file");

        // Insert lemma before the theorem
        let patch_marker = format!("theorem {}", target_theorem);
        let patched_content = content.replace(
            &patch_marker,
            &format!(
                "/-- Synthesized by NPE-Rust --/\n{}\n\ntheorem {}",
                lemma_body, target_theorem
            ),
        );

        // Update the proof to USE the new lemma
        let final_content = patched_content.replace(
            "sorry",
            "exact sub_eq_of_add_eq h1' (hl.trans_le (by simp))", // This is a heuristic guess at the proof
        );

        std::fs::write(&target_file, final_content).expect("Failed to patch Lean file");
        println!("Patch applied to {}", target_file.display());

        // 5. [NPE-Lean] Verify the improved loop
        println!("\n[NPE-Lean] Verifying proof with improved search engine...");
        let output = Command::new(&advisor.lake_cmd)
            .args(["build", "Coh.Boundary.FieldEquation"])
            .current_dir(&project_path)
            .output()
            .expect("Failed to execute lake");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        if let Some(report) = generate_failure_report("rust-adv-lean-1", target_theorem, &combined)
        {
            println!("RESULT: Attempt failed, but diagnostic ingested.");
            println!("Layer: {:?}, Kind: {:?}", report.layer, report.kind);

            // Ingest into PhaseLoom
            let summary = BoundaryReceiptSummary {
                domain: "code".to_string(),
                strategy_class: "synthesize".to_string(),
                target: target_theorem.to_string(),
                accepted: false,
                novelty: 0.8,
                failure_report: Some(report),
                ..Default::default()
            };
            phaseloom_ingest(&mut phaseloom, &summary, &config);
        } else {
            println!("RESULT: SUCCESS! The proof was closed by the synthesized lemma.");

            let summary = BoundaryReceiptSummary {
                domain: "code".to_string(),
                strategy_class: "synthesize".to_string(),
                target: target_theorem.to_string(),
                accepted: true,
                novelty: 1.0,
                ..Default::default()
            };
            phaseloom_ingest(&mut phaseloom, &summary, &config);
        }
    } else {
        println!(
            "FAILURE: Synthesizer could not handle pattern '{}'",
            missing_pattern
        );
    }

    println!("\nFinal PhaseLoom Weights: {:?}", phaseloom.all_weights());
}

