use crate::equivalence_hunter::{EquivalenceDiagnosis, EquivalenceHunter};
use crate::failure_memory::FailureMemory;
use crate::invariant_hunter::{InvariantDiagnosis, InvariantHunter};
use crate::lean_error::{classify_lean_error, LeanErrorKind};
use crate::lean_worker::LeanWorker;
use crate::lemma_forge::{DerivationPlan, LemmaForge};
use crate::npe::tools::ctrl_cohbit_adapter::{
    attempt_to_cohbit, build_receipt, verify_receipt_integrity, CtrlAccountingBudget,
    CtrlCohBitCandidate, CtrlCohTrajectory, CtrlObjectiveResult, CtrlRepairReceipt,
};
use crate::repair::{choose_repair_action, RepairAction};
use crate::theorem_state::{capture_post_state, capture_pre_state, derive_chain_inputs};
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
    // NEW: CohBit integration fields
    pub cohbit_candidate: Option<CtrlCohBitCandidate>,
    pub cohbit_receipt: Option<CtrlRepairReceipt>,
}

pub struct CtrlLoop {
    pub project_path: PathBuf,
    pub worker: LeanWorker,
    pub memory: FailureMemory,
    // NEW: Track CohBit trajectory across repairs
    pub trajectory: Option<CtrlCohTrajectory>,
}

impl CtrlLoop {
    pub fn new(project_path: PathBuf) -> Result<Self, String> {
        let worker = LeanWorker::start(&project_path, "lake")?;
        Ok(Self {
            project_path,
            worker,
            memory: FailureMemory::new(),
            trajectory: None,
        })
    }

    pub fn new_with_cmd(project_path: PathBuf, lake_cmd: &str) -> Result<Self, String> {
        let worker = LeanWorker::start(&project_path, lake_cmd)?;
        Ok(Self {
            project_path,
            worker,
            memory: FailureMemory::new(),
            trajectory: None,
        })
    }

    /// Attempts to repair a theorem by trying a list of candidate tactics.
    pub fn repair_theorem(
        &mut self,
        theorem: &str,
        candidates: Vec<&str>,
    ) -> Result<CtrlResult, String> {
        // Initialize trajectory if needed
        if self.trajectory.is_none() {
            self.trajectory = Some(CtrlCohTrajectory::default());
        }

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
                    cohbit_candidate: None,
                    cohbit_receipt: None,
                });
            }

            // 2. Check failure memory first
            if self.memory.has_failed(theorem, tactic) {
                continue;
            }

            if let Ok(res) = self.worker.try_tactic(theorem, tactic) {
                if res["result"] == "success" {
                    let proof_hash = res["proof_hash"].as_str().unwrap_or("0x0").to_string();

                    // NEW: Build CohBit candidate from successful repair
                    let (cohbit_candidate, cohbit_receipt) =
                        self.build_cohbit_from_repair(theorem, tactic, &proof_hash);

                    // Add to trajectory
                    if let Some(ref mut traj) = self.trajectory {
                        if let Some(ref candidate) = cohbit_candidate {
                            let objective = CtrlObjectiveResult::LeanAccepted {
                                theorem_name: theorem.to_string(),
                                proof_hash: proof_hash.clone(),
                                stdout_hash: "".to_string(),
                                stderr_hash: "".to_string(),
                            };
                            traj.add_attempt(
                                theorem.to_string(),
                                candidate.receipt.candidate_hash.clone(),
                                &objective,
                            );
                        }
                    }

                    return Ok(CtrlResult {
                        theorem: theorem.to_string(),
                        tactic: tactic.to_string(),
                        proof_hash,
                        success: true,
                        error_kind: None,
                        invariant_diagnosis: None,
                        equivalence_diagnosis: None,
                        derivation_plan: None,
                        cohbit_candidate,
                        cohbit_receipt,
                    });
                } else {
                    let stderr = res["stderr"].as_str().unwrap_or("");
                    let kind = classify_lean_error(stderr);

                    // Invariant/Equivalence Hunter integration - NOW USED
                    let diagnosis = InvariantHunter::hunt(theorem, "", stderr);
                    let eq_diagnosis = EquivalenceHunter::hunt(theorem, "");

                    // Lemma Forge integration - NOW USED
                    let forge_plan = LemmaForge::plan(theorem, stderr);

                    // Generate candidate tactics from diagnostics
                    let candidate_tactics =
                        Self::generate_candidates(theorem, &diagnosis, &eq_diagnosis, &forge_plan);

                    // Try diagnostic-derived candidates before giving up
                    for tactic in candidate_tactics {
                        if let Ok(res) = self.worker.try_tactic(theorem, &tactic) {
                            if res["result"] == "success" {
                                return Ok(CtrlResult {
                                    theorem: theorem.to_string(),
                                    tactic: tactic,
                                    proof_hash: res["proof_hash"]
                                        .as_str()
                                        .unwrap_or("0x0")
                                        .to_string(),
                                    success: true,
                                    error_kind: None,
                                    invariant_diagnosis: Some(diagnosis),
                                    equivalence_diagnosis: Some(eq_diagnosis),
                                    derivation_plan: Some(forge_plan),
                                    cohbit_candidate: None,
                                    cohbit_receipt: None,
                                });
                            }
                        }
                    }

                    self.memory.record_failure(
                        theorem.to_string(),
                        tactic.to_string(),
                        kind,
                        stderr.to_string(),
                    );
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
            cohbit_candidate: None,
            cohbit_receipt: None,
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
            Ok(format!(
                "FAILURE: Kind={:?}, Action={:?}, Error={}",
                kind, action, stderr
            ))
        }
    }

    /// Attempts to repair a theorem and validates the result against the full build.
    pub fn repair_and_verify(
        &mut self,
        theorem: &str,
        candidates: Vec<&str>,
    ) -> Result<CtrlResult, String> {
        let res = self.repair_theorem(theorem, candidates)?;
        if res.success {
            // Validate that the fix doesn't break the build
            let build_res = self.audit_repo()?;
            if build_res == "SUCCESS" {
                Ok(res)
            } else {
                Err(format!(
                    "Fix applied locally but build failed: {}",
                    build_res
                ))
            }
        } else {
            Ok(res)
        }
    }

    /// Generate candidate tactics from diagnostics
    fn generate_candidates(
        theorem: &str,
        inv_diag: &crate::invariant_hunter::InvariantDiagnosis,
        eq_diag: &crate::equivalence_hunter::EquivalenceDiagnosis,
        forge_plan: &crate::lemma_forge::DerivationPlan,
    ) -> Vec<String> {
        let mut candidates = Vec::new();

        // Add tactics from LemmaForge proof strategy
        for tactic in &forge_plan.proof_strategy {
            candidates.push(tactic.clone());
        }

        // Add tactics from EquivalenceHunter proof strategy
        if let Some(ref strategy) = eq_diag.proof_strategy {
            candidates.push(strategy.clone());
        }

        // Add suggested lemmas as exact tactics
        for lemma in &forge_plan.suggested_lemmas {
            candidates.push(format!("exact {}", lemma));
        }

        // Add known lemmas from invariant diagnosis
        for lemma in &inv_diag.suggested_lemmas {
            candidates.push(format!("exact {}", lemma));
        }

        // Add safe fallback tactics based on missing invariants
        for inv in &inv_diag.missing {
            match *inv {
                crate::invariant_hunter::InvariantKind::CommitInequality => {
                    candidates.push("linarith".to_string());
                }
                crate::invariant_hunter::InvariantKind::LorentzInvariance => {
                    candidates.push("ring".to_string());
                }
                _ => {
                    candidates.push("simp".to_string());
                }
            }
        }

        candidates
    }

    /// Output repair audit to JSON for inspection
    pub fn audit_trail(result: &CtrlResult) -> String {
        let json = serde_json::json!({
            "theorem": result.theorem,
            "tactic": result.tactic,
            "proof_hash": result.proof_hash,
            "success": result.success,
            "error_kind": result.error_kind,
            "invariant_diagnosis": result.invariant_diagnosis,
            "equivalence_diagnosis": result.equivalence_diagnosis,
            "derivation_plan": result.derivation_plan,
            // NEW: Include CohBit data in audit trail
            "cohbit_receipt": result.cohbit_receipt.as_ref().map(|r| {
                serde_json::json!({
                    "theorem_name": r.theorem_name,
                    "theorem_hash_pre": r.theorem_hash_pre,
                    "theorem_hash_post": r.theorem_hash_post,
                    "spend": r.spend,
                    "defect_reserve": r.defect_reserve,
                    "authority": r.authority,
                })
            }),
            "cohbit_candidate": result.cohbit_candidate.is_some(),
        });

        serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
    }

    /// Build CohBit candidate from successful repair with FULL CORE VERIFICATION
    /// This bridges CTRL repair results to CohBit verification
    /// Enforces: accounting admissibility + V3 core verification = CertifiedCohBit
    fn build_cohbit_from_repair(
        &mut self,
        theorem: &str,
        tactic: &str,
        proof_hash: &str,
    ) -> (Option<CtrlCohBitCandidate>, Option<CtrlRepairReceipt>) {
        // Get current trajectory index
        let step_index = self
            .trajectory
            .as_ref()
            .map(|t| t.attempts.len() as u64)
            .unwrap_or(0);

        // Get theorem file path - construct from project_path and theorem name
        let theorem_file = self.project_path.join(format!("{}.lean", theorem));

        // === CANONICAL THEOREM STATE COMMITMENT ===
        // Use filesystem snapshots instead of synthetic hashes
        let theorem_hash_pre = match capture_pre_state(theorem, &theorem_file) {
            Ok(hash) => hash,
            Err(e) => {
                tracing::warn!("failed to capture pre state: {}", e);
                // Fallback to synthetic hash if filesystem unavailable
                format!("pre_{}", theorem)
            }
        };

        let theorem_hash_post = match capture_post_state(theorem, &theorem_file, proof_hash) {
            Ok(hash) => hash,
            Err(e) => {
                tracing::warn!("failed to capture post state: {}", e);
                // Fallback to synthetic hash if filesystem unavailable
                format!("post_{}_{}", theorem, proof_hash)
            }
        };

        let tactic_hash = format!("tactic_{}", tactic);
        let candidate_hash = format!("candidate_{}_{}", theorem, proof_hash);

        // Build accounting from repair
        let mut accounting = CtrlAccountingBudget::initial(1000, 100);
        accounting.with_tactic_cost(50); // Assume base cost for a tactic
        accounting.with_post_confidence(900);

        // Build objective result
        let objective = CtrlObjectiveResult::LeanAccepted {
            theorem_name: theorem.to_string(),
            proof_hash: proof_hash.to_string(),
            stdout_hash: "".to_string(),
            stderr_hash: "".to_string(),
        };

        // Build receipt
        let receipt = build_receipt(
            theorem.to_string(),
            theorem_hash_pre.clone(),
            theorem_hash_post.clone(),
            candidate_hash.clone(),
            tactic_hash.clone(),
            proof_hash.to_string(),
            "audit".to_string(),
            accounting.spend as u128,
            accounting.defect as u128,
            accounting.authority as u128,
            format!("acc_{}", step_index),
        );

        // === FULL V3 CORE VERIFICATION ===
        // Convert to V3 wire and verify with coh-core
        use coh_npe::tools::ctrl_cohbit_adapter::verify_receipt_full;
        let verification_result = verify_receipt_full(&receipt, &accounting, step_index);

        // Only emit CertifiedCohBit if core verifier accepts (using pattern match)
        if !matches!(
            verification_result.decision,
            coh_core::types::Decision::Accept
        ) {
            // Core verifier rejected - do NOT emit candidate
            return (None, Some(receipt));
        }

        // Create candidate (now fully certified)
        let candidate = attempt_to_cohbit(&objective, &receipt, &accounting);

        (candidate, Some(receipt))
    }
}
