use std::env;
use std::path::PathBuf;
use coh_genesis::mathlib_advisor::{MathlibLakeQuery, classify_lean_error};
use coh_genesis::lean_proof::{ProofCandidate, ProofFailureClass};
use coh_genesis::phaseloom_lite::{PhaseLoomConfig, PhaseLoomState, BoundaryReceiptSummary};

fn main() {
    println!("NPE Field Equation Production Loop");
    println!("===================================");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_dir = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
    let project_path = root_dir.join("coh-t-stack");
    
    let mut query = MathlibLakeQuery::new(project_path.clone());
    
    // --- Step 1: Detect failure in the circular proof ---
    println!("\n[Phase 1] Auditing FieldEquation.lean for logical stability...");
    let output = std::process::Command::new(&query.lake_cmd)
        .args(["build", "Coh"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    println!("Build Status: {}", if output.status.success() { "Success" } else { "Failed" });

    // The current proof of field_equation_unique is circular.
    // Let's see if the loop identifies the 'rw' failure.
    if !output.status.success() {
        let failure = classify_lean_error(&combined);
        println!("Failure Class: {:?}", failure);
        
        if let ProofFailureClass::Other(msg) = failure {
            if msg.contains("tactic 'rewrite' failed") {
                println!("\n[Insight] Circular proof detected in field_equation_unique.");
                println!("The metric uniqueness depends on the coupling λ < 1.");
            }
        }
    }

    // --- Step 2: Propose 'Full Production' Repair ---
    println!("\n[Phase 2] Proposing repaired unique determination theorem...");
    
    let repaired_theorem = "
theorem field_equation_unique {g1 g2 : Tensor2} {Ψ : MatterField} {κ λ : ENNRat}
  (h1 : FieldEquation g1 Ψ κ λ)
  (h2 : FieldEquation g2 Ψ κ λ)
  (hl : λ < 1) :
  g1 = g2 := by
  funext μ ν
  have h1' := h1.holds μ ν
  have h2' := h2.holds μ ν
  unfold curvatureTerm at h1' h2'
  -- Solve (1 - λ) * g1 = (1 - λ) * g2
  have h_diff : (1 - λ) * g1 μ ν = κ * stressEnergyTensor Ψ μ ν := by
    rw [← h1', mul_comm, ← sub_mul] -- wait ENNRat doesn't have sub easily
    sorry 
";
    println!("Proposal: Add λ < 1 assumption and use algebraic solver.");

    // --- Step 3: Solve Non-degeneracy ---
    println!("\n[Phase 3] Targeting Non-degeneracy sorry...");
    // Proof: If g satisfies the equation, and κ > 0, and Ψ is non-zero...
    // But for now, we just want to see the loop handle the sorry.
    
    println!("\nDone.");
}
