use coh_genesis::ctrl::CtrlLoop;
use std::path::PathBuf;

fn main() {
    println!("--- Phase 4: Autonomous Relativistic Repair Batch ---");

    let project_path = PathBuf::from("../../../coh-t-stack");
    let mut ctrl = match CtrlLoop::new_with_cmd(project_path, "C:\\Users\\truea\\.elan\\bin\\lake.exe") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to initialize CTRL loop: {}", e);
            return;
        }
    };

    println!("\n[STEP 1] Auditing Repository...");
    let audit_res = match ctrl.audit_repo() {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Audit failed: {}", e);
            return;
        }
    };

    if audit_res == "SUCCESS" {
        println!("[SUCCESS] All modules verified. Zero-Sorry Kernel achieved.");
    } else {
        println!("[ANALYSIS] Build failure detected. Applying v1.1 Taxonomy...");
        println!("{}", audit_res);
        
        // Example: Attempt repair on a known difficult theorem if it's in the failure set
        if audit_res.contains("trajectory_commit_telescopes") {
            println!("\n[REPAIR] Attempting targeted repair on 'trajectory_commit_telescopes'...");
            let repair_res = ctrl.repair_and_verify(
                "trajectory_commit_telescopes",
                vec![
                    "induction τ.actions generalizing τ.states; simp; match τ.states with | [x] => simp; exact le_refl _ | _ => contradiction",
                    "induction τ.actions; simp_all",
                ]
            );
            println!("Repair Result: {:?}", repair_res);
        }
    }
}
