use crate::lean_worker::LeanWorker;
use crate::lean_error::{classify_lean_error, LeanErrorKind};
use crate::repair::{choose_repair_action, RepairAction};
use crate::invariant_hunter::{InvariantHunter, InvariantDiagnosis};
use crate::equivalence_hunter::{EquivalenceHunter, EquivalenceDiagnosis};
use crate::lemma_forge::{LemmaForge, DerivationPlan};
use crate::failure_memory::FailureMemory;
use std::path::PathBuf;

#[derive(Debug)]
pub struct CtrlResult {
    pub theorem: String,
    pub tactic: String,
    pub proof_hash: String,
    pub success: bool,
    pub error_kind: Option<LeanErrorKind>,
    pub invariant_diagnosis: Option<InvariantDiagnosis>,
    pub equivalence_diagnosis: Option<EquivalenceDiagnosis>,
    pub derivation_plan: Option<DerivationPlan>,
}

pub struct CtrlLoop {
    pub project_path: PathBuf,
    pub worker: LeanWorker,
    pub memory: FailureMemory,
}

impl CtrlLoop {
    pub fn new(project_path: PathBuf) -> Result<Self, String> {
        let worker = LeanWorker::start(&project_path, "lake")?;
        Ok(Self { project_path, worker, memory: FailureMemory::new() })
    }

    pub fn new_with_cmd(project_path: PathBuf, lake_cmd: &str) -> Result<Self, String> {
        let worker = LeanWorker::start(&project_path, lake_cmd)?;
        Ok(Self { project_path, worker, memory: FailureMemory::new() })
    }

    /// Attempts to repair a theorem by trying a list of candidate tactics.
    pub fn repair_theorem(&mut self, theorem: &str, candidates: Vec<&str>) -> Result<CtrlResult, String> {
        for tactic in candidates {
            // 1. Check for forbidden shortcuts (No-Bluff Protocol)
            if crate::safety::contains_forbidden_shortcut(tactic) {
                return Ok(CtrlResult {
                    theorem: theorem.to_string(),
                    tactic: tactic.to_string(),
                    proof_hash: String::new(),
                    success: false,
                    error_kind: Some(crate::lean_error::LeanErrorKind::UsesForbiddenShortcut),
                    invariant_diagnosis: None,
                    equivalence_diagnosis: None,
                    derivation_plan: None,
                });
            }

            // 2. Check failure memory first
            if self.memory.has_failed(theorem, tactic) {
                continue;
            }

            if let Ok(res) = self.worker.try_tactic(theorem, tactic) {
                if res["result"] == "success" {
                    return Ok(CtrlResult {
                        theorem: theorem.to_string(),
                        tactic: tactic.to_string(),
                        proof_hash: res["proof_hash"].as_str().unwrap_or("0x0").to_string(),
                        success: true,
                        error_kind: None,
                        invariant_diagnosis: None,
                        equivalence_diagnosis: None,
                        derivation_plan: None,
                    });
                } else {
                    let stderr = res["stderr"].as_str().unwrap_or("");
                    let kind = classify_lean_error(stderr);
                    
                    // Invariant/Equivalence Hunter integration
                    let _diagnosis = InvariantHunter::hunt(theorem, "", stderr);
                    let _eq_diagnosis = EquivalenceHunter::hunt(theorem, "");
                    
                    // Lemma Forge integration
                    let _forge_plan = LemmaForge::plan(theorem, stderr);
                    
                    self.memory.record_failure(theorem.to_string(), tactic.to_string(), kind, stderr.to_string());
                }
            }
        }
        
        // Final diagnosis on complete failure
        let last_stderr = "".to_string(); 
        let diagnosis = InvariantHunter::hunt(theorem, "", &last_stderr);
        let eq_diagnosis = EquivalenceHunter::hunt(theorem, "");
        let forge_plan = LemmaForge::plan(theorem, &last_stderr);

        Ok(CtrlResult {
            theorem: theorem.to_string(),
            tactic: String::new(),
            proof_hash: String::new(),
            success: false,
            error_kind: None,
            invariant_diagnosis: Some(diagnosis),
            equivalence_diagnosis: Some(eq_diagnosis),
            derivation_plan: Some(forge_plan),
        })
    }

    /// Performs a full repository audit and returns the compiled truth or failure map.
    pub fn audit_repo(&mut self) -> Result<String, String> {
        let output = self.worker.full_build_output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if output.status.success() {
            Ok("SUCCESS".to_string())
        } else {
            let kind = classify_lean_error(&stderr);
            let action = choose_repair_action(kind);
            Ok(format!("FAILURE: Kind={:?}, Action={:?}, Error={}", kind, action, stderr))
        }
    }

    /// Attempts to repair a theorem and validates the result against the full build.
    pub fn repair_and_verify(&mut self, theorem: &str, candidates: Vec<&str>) -> Result<CtrlResult, String> {
        let res = self.repair_theorem(theorem, candidates)?;
        if res.success {
            // Validate that the fix doesn't break the build
            let build_res = self.audit_repo()?;
            if build_res == "SUCCESS" {
                Ok(res)
            } else {
                Err(format!("Fix applied locally but build failed: {}", build_res))
            }
        } else {
            Ok(res)
        }
    }
}
