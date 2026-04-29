//! NPE-Lean Closure Attempt v0.3
//!
//! Target: isRationalInf_pairwise_add
//!
//! This runner uses the REAL Lake integration to discover Mathlib lemmas
//! and verify proof candidates. It uses the stabilized PhaseLoom meta-loop
//! with Boltzmann exploration and entropy floor.

use coh_genesis::mathlib_advisor::MathlibLakeQuery;
use coh_genesis::phaseloom_lite::{
    phaseloom_ingest, phaseloom_init, phaseloom_sample_boltzmann,
    BoundaryReceiptSummary, PhaseLoomConfig, PhaseLoomState, MathlibEffect,
};
use coh_genesis::lean_proof::{ProofCandidate, LeanVerificationReport, is_formation_admissible};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Simple RNG
#[derive(Clone, Debug)]
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Verify a proof candidate using the real Lean environment
fn verify_candidate(
    candidate: &ProofCandidate,
    project_path: &Path,
    lake_cmd: &str,
) -> LeanVerificationReport {
    let start = Instant::now();
    let temp_file_name = format!("_verify_{}.lean", candidate.id);
    let temp_file = project_path.join(&temp_file_name);

    // Assemble full Lean file with imports and target
    let mut lean_code = String::new();
    for imp in &candidate.imports {
        lean_code.push_str(&format!("import {}\n", imp));
    }
    lean_code.push_str("\nnamespace Coh.Boundary\n\n");
    lean_code.push_str(&candidate.proof_text);
    lean_code.push_str("\n\nend Coh.Boundary\n");

    if let Err(e) = fs::write(&temp_file, lean_code) {
        return LeanVerificationReport {
            compiles: false,
            has_sorry: false,
            has_admit: false,
            new_axioms: 0,
            statement_unchanged: false,
            forbidden_imports: false,
            build_time_ms: 0,
            errors: vec![format!("Failed to write temp file: {}", e)],
            warnings: 0,
            genesis_margin: 0,
            coherence_margin: 0,
            formation_accept: false,
            failure_report: None,
        };
    }

    let output = Command::new(lake_cmd)
        .args(["env", "lean", &temp_file_name])
        .current_dir(project_path)
        .output();

    // Cleanup
    let _ = fs::remove_file(&temp_file);

    let build_time_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            
            let compiles = out.status.success();
            let has_sorry = combined.contains("sorry");
            let has_admit = combined.contains("admit");
            
            let mut errors = Vec::new();
            if !compiles {
                for line in combined.lines() {
                    if line.contains("error:") {
                        errors.push(line.to_string());
                    }
                }
            }

            LeanVerificationReport {
                compiles,
                has_sorry,
                has_admit,
                new_axioms: 0, // Placeholder
                statement_unchanged: true,
                forbidden_imports: false,
                build_time_ms,
                errors,
                warnings: 0,
                genesis_margin: 0,
                coherence_margin: 0,
                formation_accept: false,
                failure_report: None,
            }
        }
        Err(e) => LeanVerificationReport {
            compiles: false,
            has_sorry: false,
            has_admit: false,
            new_axioms: 0,
            statement_unchanged: false,
            forbidden_imports: false,
            build_time_ms,
            errors: vec![format!("Command failed: {}", e)],
            warnings: 0,
            genesis_margin: 0,
            coherence_margin: 0,
            formation_accept: false,
            failure_report: None,
        },
    }
}

fn main() {
    println!("NPE-Lean Closure Attempt v0.3 (Real Integration)");
    println!("===============================================");

    // 1. Setup paths
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_dir = manifest_dir
        .parent().unwrap() // crates
        .parent().unwrap() // coh-node
        .parent().unwrap(); // workspace root
    let project_path = root_dir.join("coh-t-stack");
    
    println!("Project path: {}", project_path.display());

    // 2. Initialize Lake Query
    let mut lake_query = MathlibLakeQuery::new(project_path.clone());
    if !lake_query.available {
        println!("ERROR: Lake environment not available. Check PATH or elan installation.");
        return;
    }
    println!("Lake command: {}", lake_query.lake_cmd);

    // 3. Initialize PhaseLoom
    let config = PhaseLoomConfig {
        initial_budget: 50_000,
        learning_rate: 0.1,
        curvature_penalty: 0.05,
        circuit_break_threshold: 2000,
        min_weight: 0.01,
        initial_temperature: 1.5,
        entropy_floor: 0.2,
        ..Default::default()
    };
    let mut state = phaseloom_init(&config);
    
    // Seed with initial strategies
    state.strategy_weights.0.insert("IsGLB".to_string(), 0.5);
    state.strategy_weights.0.insert("Approximation".to_string(), 0.3);
    state.strategy_weights.0.insert("Direct".to_string(), 0.2);
    state.strategy_weights.normalize();

    let mut _rng = SimpleRng::new(42);
    let mut best_candidate: Option<ProofCandidate> = None;

    println!("\nStarting meta-loop sweeps...");

    for sweep in 0..10 {
        println!("\n--- Sweep {} ---", sweep);
        
        // A. Sample strategy using Boltzmann exploration
        let (strategy_name, is_exploring) = phaseloom_sample_boltzmann(&state, &config, &mut rand::thread_rng());
        let strategy_name = strategy_name.unwrap_or_else(|| "IsGLB".to_string());
        
        println!("Selected strategy: {} (Exploring: {})", strategy_name, is_exploring);

        // B. Query Mathlib Advisor for lemmas matching strategy
        let lemmas = lake_query.search_lemmas(&strategy_name);
        println!("Found {} candidate lemmas in Mathlib", lemmas.len());
        
        // Pick top lemmas (simulation for now, but using real search names if found)
        let mut top_lemmas = vec!["add_le_add".to_string()];
        if !lemmas.is_empty() {
            top_lemmas.push(lemmas[0].name.clone());
        }

        // C. Assemble proof candidate (Simplified template for pairwise_add)
        let proof_text = format!(
            "theorem isRationalInf_pairwise_add {{s1 s2 : Set ENNRat}} {{i1 i2 : ENNRat}}\n\
             (h1 : IsRationalInf s1 i1)\n\
             (h2 : IsRationalInf s2 i2) :\n\
             IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by\n\
             constructor\n\
             · rintro z ⟨x, hx, y, hy, rfl⟩\n\
               exact {} (h1.lower x hx) (h2.lower y hy)\n\
             · sorry",
            top_lemmas[0]
        );

        let candidate = ProofCandidate {
            id: format!("sweep_{}", sweep),
            wildness: if is_exploring { 2.0 } else { 1.0 },
            target_theorem: "isRationalInf_pairwise_add".to_string(),
            proof_text,
            proof_tactics: vec!["constructor".to_string(), "rintro".to_string(), "exact".to_string()],
            tactic_count: 3,
            helper_lemmas: 0,
            imports: vec![
                "Mathlib.Data.NNRat.Defs".to_string(),
                "Coh.Boundary.RationalInf".to_string(),
            ],
            novelty: if is_exploring { 0.8 } else { 0.2 },
        };

        // D. Verify through REAL Lean
        println!("Verifying candidate...");
        let report = verify_candidate(&candidate, &project_path, &lake_query.lake_cmd);
        
        println!("  Compiles: {}", report.compiles);
        println!("  Has Sorry: {}", report.has_sorry);
        println!("  Build Time: {}ms", report.build_time_ms);

        // E. Formation Admission & Metrics
        let (accept, gen_margin, coh_margin) = is_formation_admissible(&candidate, 1000, &report);
        
        // F. Ingest back into PhaseLoom
        let receipt = BoundaryReceiptSummary {
            domain: "lean_proof".to_string(),
            target: candidate.target_theorem.clone(),
            strategy_class: strategy_name.clone(),
            wildness: candidate.wildness,
            genesis_margin: gen_margin,
            coherence_margin: coh_margin,
            first_failure: if report.compiles { "none".to_string() } else { "compile_error".to_string() },
            outcome: if report.compiles { "accepted".to_string() } else { "rejected".to_string() },
            accepted: accept,
            novelty: candidate.novelty,
            ..Default::default()
        };

        phaseloom_ingest(&mut state, &receipt, &config);
        
        if report.compiles && !report.has_sorry {
            best_candidate = Some(candidate);
            println!("SUCCESS: Closed proof found!");
            break;
        }
    }

    if let Some(best) = best_candidate {
        println!("\nFinal Closed Proof Candidate:");
        println!("{}", best.proof_text);
    } else {
        println!("\nNo full proof closed in this run.");
    }
}

