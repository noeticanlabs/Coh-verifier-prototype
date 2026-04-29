use std::env;
use std::path::PathBuf;
use coh_genesis::mathlib_advisor::{MathlibLakeQuery, generate_failure_report};
use coh_genesis::failure_taxonomy::{FailureKind, LeanElabFailure};

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
    let report = generate_failure_report("example", "RationalInf", &combined);
    
    if let Some(r) = report {
        println!("Detected Failure: {:?}", r.kind);

        match r.kind {
            FailureKind::LeanElab(LeanElabFailure::UnknownIdentifier(id)) => {
                println!("\n[Phase 2] Repairing unknown identifier: '{}'...", id);
                println!("Searching Mathlib for '{}'...", id);
                let results = query.search_lemmas(&id);
                
                if let Some(best) = results.first() {
                    println!("Found lemma: {} in file {}", best.name, best.file);
                    println!("Recommended Action: Check the file path to determine the import.");
                }
            }
            FailureKind::LeanElab(LeanElabFailure::FailedToSynthesizeInstance(inst)) => {
                println!("\n[Phase 2] Repairing missing instance: '{}'...", inst);
                println!("This usually requires more specific imports or 'open' statements.");
            }
            _ => {
                println!("Unrecognized failure type for this example.");
            }
        }
    } else {
        println!("No failures detected.");
    }

    println!("\nRepair Demo Complete.");
}
