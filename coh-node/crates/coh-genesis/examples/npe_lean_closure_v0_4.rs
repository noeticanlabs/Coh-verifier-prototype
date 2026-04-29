use std::env;
use std::path::PathBuf;
use coh_genesis::mathlib_advisor::{MathlibLakeQuery, generate_failure_report};
use coh_genesis::failure_taxonomy::{FailureKind, LeanElabFailure};
use coh_genesis::phaseloom_lite::{PhaseLoomConfig, PhaseLoomState, BoundaryReceiptSummary};

fn main() {
    println!("NPE-Lean Intelligent Failure Recovery v0.4");
    println!("==========================================");

    // 1. Setup paths
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_dir = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
    let project_path = root_dir.join("coh-t-stack");
    
    let mut query = MathlibLakeQuery::new(project_path.clone());
    if !query.available {
        println!("Lake not available, skipping real integration demo.");
        return;
    }

    let config = PhaseLoomConfig::default();
    let mut state = PhaseLoomState::new(&config);

    // --- Step 1: Initial Attempt (Deliberate Failure) ---
    println!("\n[Attempt 1] Proposing proof with intentional 'unknown identifier' error...");
    
    let bad_lemma = "add_le_add_typo";
    let novelty = 0.9;

    // Simulate verification (using real toolchain to get the real error message)
    let temp_file = project_path.join("_repair_temp.lean");
    let lean_code = format!(
        "import Mathlib.Order.Basic\nimport Coh.Boundary.RationalInf\nopen Coh.Boundary\ntheorem repair_test (a b c d : NNRat) : a + b ≤ c + d := by apply {}\n",
        bad_lemma
    );
    std::fs::write(&temp_file, lean_code).unwrap();

    let output = std::process::Command::new(&query.lake_cmd)
        .args(["env", "lean", "_repair_temp.lean"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    println!("Actual Output: {}", combined);
    
    // Classify error
    let report = generate_failure_report("repair-demo-1", "repair_test", &combined);
    
    if let Some(r) = report {
        println!("Detected Failure Kind: {:?}", r.kind);

        // Ingest into PhaseLoom
        let receipt = BoundaryReceiptSummary {
            strategy_class: "Approximation".to_string(),
            accepted: false,
            novelty,
            failure_report: Some(r.clone()),
            ..Default::default()
        };
        state.ingest(&receipt, &config);
        
        println!("PhaseLoom updated. Current weight for 'Approximation': {:.4}", state.weight_for("Approximation"));

        // --- Step 2: Repair Loop ---
        if let FailureKind::LeanElab(LeanElabFailure::UnknownIdentifier(ref id)) = r.kind {
            println!("\n[Repair] 'Unknown identifier' detected: '{}'. Triggering Mathlib search...", id);
            
            // Strategy: Search for a similar pattern
            let search_query = "(_ + _ ≤ _ + _)"; // Pattern-based repair
            println!("Searching Mathlib for pattern '{}'...", search_query);
            let results = query.search_lemmas(search_query);
            
            // Filter for something that looks like 'add_le_add'
            let best_match = results.iter()
                .find(|m| m.name.contains("add_le_add"))
                .or_else(|| results.first());

            if let Some(best_match) = best_match {
                println!("Found candidate repair lemma: '{}'", best_match.name);
                
                // --- Step 3: Second Attempt (Successful) ---
                println!("\n[Attempt 2] Retrying proof with repaired lemma...");
                
                // Re-verify
                let lean_code_2 = format!(
                    "import Mathlib.Order.Basic\nimport Coh.Boundary.RationalInf\nopen Coh.Boundary\ntheorem repair_test (a b c d : NNRat) : a + b ≤ c + d := by apply {}\n",
                    best_match.name
                );
                std::fs::write(project_path.join("_repair_temp_2.lean"), lean_code_2).unwrap();
                let output_2 = std::process::Command::new(&query.lake_cmd)
                    .args(["env", "lean", "_repair_temp_2.lean"])
                    .current_dir(&project_path)
                    .output()
                    .unwrap();
                let _ = std::fs::remove_file(project_path.join("_repair_temp_2.lean"));

                let out_2_combined = format!("{}{}", String::from_utf8_lossy(&output_2.stdout), String::from_utf8_lossy(&output_2.stderr));
                println!("Attempt 2 Output: {}", out_2_combined);

                if output_2.status.success() {
                    println!("SUCCESS: Repaired proof compiles!");
                    let success_receipt = BoundaryReceiptSummary {
                        strategy_class: "Approximation".to_string(),
                        accepted: true,
                        novelty: 1.0,
                        ..Default::default()
                    };
                    state.ingest(&success_receipt, &config);
                    println!("PhaseLoom weight for 'Approximation' recovered: {:.4}", state.weight_for("Approximation"));
                } else {
                    println!("FAILED: Repair attempt also failed.");
                }
            }
        }
    } else {
        println!("No failures detected.");
    }

    let _ = std::fs::remove_file(temp_file);
    println!("\nDemo Complete.");
}
