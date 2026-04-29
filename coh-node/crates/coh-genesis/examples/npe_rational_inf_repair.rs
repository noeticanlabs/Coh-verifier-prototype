use std::env;
use std::path::PathBuf;
use coh_genesis::mathlib_advisor::{MathlibLakeQuery, classify_lean_error};
use coh_genesis::lean_proof::{ProofCandidate, ProofFailureClass};

fn main() {
    println!("NPE RationalInf Repair Loop");
    println!("===========================");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_dir = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
    let project_path = root_dir.join("coh-t-stack");
    
    let mut query = MathlibLakeQuery::new(project_path.clone());
    
    println!("\n[Phase 1] Capturing live failures in RationalInf.lean...");
    let output = std::process::Command::new(&query.lake_cmd)
        .args(["build", "Coh"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    
    // Classify errors
    let failure_class = classify_lean_error(&combined);
    println!("Detected Failure: {:?}", failure_class);

    if let ProofFailureClass::UnknownIdentifier(id) = failure_class {
        println!("\n[Phase 2] Repairing unknown identifier: '{}'...", id);
        println!("Searching Mathlib for '{}'...", id);
        let results = query.search_lemmas(&id);
        
        if let Some(best) = results.first() {
            println!("Found lemma: {} in file {}", best.name, best.file);
            println!("Recommended Action: Check the file path to determine the import.");
        }
    } else if let ProofFailureClass::InstanceMissing(inst) = failure_class {
        println!("\n[Phase 2] Repairing missing instance: '{}'...", inst);
        println!("This usually requires more specific imports or 'open' statements.");
    }

    println!("\nRepair Demo Complete.");
}
