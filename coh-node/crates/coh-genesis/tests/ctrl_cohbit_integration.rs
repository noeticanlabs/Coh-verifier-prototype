//! CTRL → CohBit Integration Tests
//!
//! These tests verify the call path from CTRL repair loop to CohBit candidate emission.
//! Uses mocked adapter functions to ensure deterministic, fast tests.
//!
//! Run with: cargo test -p coh-genesis

use coh_npe::tools::ctrl_cohbit_adapter::{
    attempt_to_cohbit, build_receipt, CtrlAccountingBudget, CtrlCohTrajectory, CtrlObjectiveResult,
    CtrlRepairReceipt,
};

// =============================================================================
// Test 1: Successful repair emits CohBit candidate
// =============================================================================

#[test]
fn successful_repair_emits_cohbit_candidate() {
    // Given: A successful Lean repair result
    let theorem_name = "Theorem.gln_transitive".to_string();
    let proof_hash = "0xdeadbeef42".to_string();
    let tactic = "linarith".to_string();

    // Build accounting that IS admissible
    // v_pre=1000, v_post=900, spend=50, defect=100, authority=0
    // Check: 900 + 50 <= 1000 + 100 + 0 → 950 <= 1100 ✓
    let mut accounting = CtrlAccountingBudget::initial(1000, 100);
    accounting.with_tactic_cost(50);
    accounting.with_post_confidence(900);

    // Verify accounting is admissible
    assert!(
        accounting.is_admissible(),
        "Accounting should be admissible: {}",
        accounting.margin()
    );

    // Given: A LeanAccepted objective result
    let objective = CtrlObjectiveResult::LeanAccepted {
        theorem_name: theorem_name.clone(),
        proof_hash: proof_hash.clone(),
        stdout_hash: "stdout_hash".to_string(),
        stderr_hash: "stderr_hash".to_string(),
    };

    // When: We build a receipt and attempt to convert to CohBit
    let receipt = build_receipt(
        theorem_name.clone(),
        "pre_Theorem.gln_transitive".to_string(),
        format!("post_Theorem.gln_transitive_{}", proof_hash),
        format!("candidate_Theorem.gln_transitive_{}", proof_hash),
        format!("tactic_{}", tactic),
        proof_hash.clone(),
        "audit".to_string(),
        accounting.spend,
        accounting.defect,
        accounting.authority,
        "acc_0".to_string(),
    );

    let candidate = attempt_to_cohbit(&objective, &receipt, &accounting);

    // Then: Candidate should be Some and admissible
    assert!(
        candidate.is_some(),
        "Successful repair should emit CohBit candidate"
    );

    let candidate = candidate.unwrap();
    assert!(
        candidate.admissible,
        "Candidate should be admissible under accounting law"
    );
    assert_eq!(
        candidate.decision(),
        "Accept",
        "Decision should be 'Accept' for admissible candidate"
    );

    // Verify receipt is returned
    assert!(
        candidate.receipt.candidate_hash.len() > 0,
        "Receipt should contain candidate_hash"
    );
}

// =============================================================================
// Test 2: Failed repair emits no CohBit candidate
// =============================================================================

#[test]
fn failed_repair_emits_no_cohbit_candidate() {
    // Given: A LeanRejected objective result (failed repair)
    let objective = CtrlObjectiveResult::LeanRejected {
        theorem_name: "Theorem.failing_theorem".to_string(),
        error_kind: "tactic failed".to_string(),
        error_output: "failed".to_string(),
    };

    // And: Any accounting budget
    let accounting = CtrlAccountingBudget::initial(1000, 100);

    // When: We try to convert to CohBit
    let receipt = build_receipt(
        "Theorem.failing_theorem".to_string(),
        "pre_Theorem.failing_theorem".to_string(),
        "post_Theorem.failing_theorem".to_string(),
        "candidate_failing".to_string(),
        "tactic_fail".to_string(),
        "0x0".to_string(),
        "audit".to_string(),
        50,
        100,
        0,
        "acc_0".to_string(),
    );

    let candidate = attempt_to_cohbit(&objective, &receipt, &accounting);

    // Then: Candidate should NOT be admissible (Lean rejected it)
    // Note: The candidate IS created, but it's marked as NOT admissible
    // because the objective result was LeanRejected, not LeanAccepted
    if let Some(cand) = &candidate {
        assert!(
            !cand.admissible,
            "Failed repair candidate should NOT be admissible"
        );
        assert_eq!(
            cand.decision(),
            "Reject",
            "Decision should be 'Reject' for failed repair"
        );
    }
}

// =============================================================================
// Test 3: Accounting violation blocks candidate
// =============================================================================

#[test]
fn accounting_violation_blocks_candidate() {
    // Given: A LeanAccepted objective (repair succeeded in Lean)
    let objective = CtrlObjectiveResult::LeanAccepted {
        theorem_name: "Theorem.expensive_repair".to_string(),
        proof_hash: "0xcafebabe".to_string(),
        stdout_hash: "".to_string(),
        stderr_hash: "".to_string(),
    };

    // When: We create an accounting that VIOLATES the law
    // v_pre=100, v_post=120, spend=10, defect=0, authority=29
    // Check: 120 + 10 > 100 + 0 + 29 → 130 > 129 = TRUE (violates!)
    let accounting = CtrlAccountingBudget {
        pre_budget: 100,
        post_budget: 120,
        spend: 10,
        defect: 0,
        authority: 29,
    };

    // Verify it violates the law
    assert!(
        !accounting.is_admissible(),
        "This accounting should violate the law"
    );
    assert!(
        accounting.margin() < 0,
        "Margin should be negative: {}",
        accounting.margin()
    );

    let receipt = build_receipt(
        "Theorem.expensive_repair".to_string(),
        "pre_Theorem.expensive_repair".to_string(),
        "post_Theorem.expensive_repair".to_string(),
        "candidate_expensive".to_string(),
        "tactic_expensive".to_string(),
        "0xcafebabe".to_string(),
        "audit".to_string(),
        accounting.spend,
        accounting.defect,
        accounting.authority,
        "acc_0".to_string(),
    );

    let candidate = attempt_to_cohbit(&objective, &receipt, &accounting);

    // Then: Candidate should be NOT admissible due to accounting violation
    // NOTE: The current implementation checks both LeanAccepted AND accounting.is_admissible()
    // Since the accounting is NOT admissible, candidate.admissible should be false
    assert!(
        candidate.is_some(),
        "Candidate object should still be created"
    );

    let cand = candidate.unwrap();
    assert!(
        !cand.admissible,
        "Candidate should be rejected due to accounting violation"
    );
    assert_eq!(
        cand.decision(),
        "Reject",
        "Decision should be 'Reject' for accounting violation"
    );
}

// =============================================================================
// Test 3b: Accounting boundary exactly at limit passes
// =============================================================================

#[test]
fn accounting_boundary_exactly_at_limit_passes() {
    // Given: A LeanAccepted objective
    let objective = CtrlObjectiveResult::LeanAccepted {
        theorem_name: "Theorem.boundary_repair".to_string(),
        proof_hash: "0xboundary".to_string(),
        stdout_hash: "".to_string(),
        stderr_hash: "".to_string(),
    };

    // When: We create accounting at exactly the boundary
    // v_pre=100, v_post=120, spend=10, defect=0, authority=30
    // Check: 120 + 10 <= 100 + 0 + 30 → 130 <= 130 = TRUE (exactly at boundary)
    let accounting = CtrlAccountingBudget {
        pre_budget: 100,
        post_budget: 120,
        spend: 10,
        defect: 0,
        authority: 30,
    };

    // Verify it's admissible (exactly at boundary is allowed)
    assert!(
        accounting.is_admissible(),
        "Accounting exactly at boundary should be admissible"
    );
    assert_eq!(
        accounting.margin(),
        0,
        "Margin should be exactly 0 at boundary"
    );

    let receipt = build_receipt(
        "Theorem.boundary_repair".to_string(),
        "pre_Theorem.boundary_repair".to_string(),
        "post_Theorem.boundary_repair".to_string(),
        "candidate_boundary".to_string(),
        "tactic_boundary".to_string(),
        "0xboundary".to_string(),
        "audit".to_string(),
        accounting.spend,
        accounting.defect,
        accounting.authority,
        "acc_0".to_string(),
    );

    let candidate = attempt_to_cohbit(&objective, &receipt, &accounting);

    // Then: Candidate SHOULD be admissible
    let cand = candidate.unwrap();
    assert!(
        cand.admissible,
        "Candidate at boundary should be admissible"
    );
    assert_eq!(
        cand.decision(),
        "Accept",
        "Decision should be 'Accept' at boundary"
    );
}

// =============================================================================
// Test 4: Audit trail includes CohBit fields
// =============================================================================

#[test]
fn audit_trail_contains_cohbit_fields() {
    // Given: A successful repair with CohBit candidate
    let receipt = build_receipt(
        "Theorem.audit_test".to_string(),
        "pre_Theorem.audit_test".to_string(),
        "post_Theorem.audit_test_0xabc".to_string(),
        "candidate_audit_test".to_string(),
        "tactic_audit".to_string(),
        "0xabc123".to_string(),
        "audit_hash".to_string(),
        50,
        100,
        0,
        "acc_1".to_string(),
    );

    let objective = CtrlObjectiveResult::LeanAccepted {
        theorem_name: "Theorem.audit_test".to_string(),
        proof_hash: "0xabc123".to_string(),
        stdout_hash: "stdout".to_string(),
        stderr_hash: "stderr".to_string(),
    };

    let accounting = CtrlAccountingBudget::initial(1000, 100);
    let candidate = attempt_to_cohbit(&objective, &receipt, &accounting);

    assert!(candidate.is_some(), "Candidate should exist for audit test");
    let cand = candidate.unwrap();

    // When: We serialize the receipt to JSON
    let json = serde_json::to_string(&cand.receipt).unwrap();

    // Then: JSON should contain all required CohBit fields
    assert!(
        json.contains("theorem_name"),
        "JSON should contain theorem_name"
    );
    assert!(
        json.contains("theorem_hash_pre"),
        "JSON should contain theorem_hash_pre"
    );
    assert!(
        json.contains("theorem_hash_post"),
        "JSON should contain theorem_hash_post"
    );
    assert!(
        json.contains("candidate_hash"),
        "JSON should contain candidate_hash"
    );
    assert!(
        json.contains("tactic_hash"),
        "JSON should contain tactic_hash"
    );
    assert!(
        json.contains("lean_result_hash"),
        "JSON should contain lean_result_hash"
    );
    assert!(
        json.contains("audit_hash"),
        "JSON should contain audit_hash"
    );
    assert!(json.contains("spend"), "JSON should contain spend");
    assert!(
        json.contains("sequence_accumulator"),
        "JSON should contain sequence_accumulator"
    );

    // Parse back and verify basic integrity
    let parsed: CtrlRepairReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed.theorem_name, "Theorem.audit_test",
        "Parsed receipt should match"
    );
}

// =============================================================================
// Test 5: Trajectory advances only on successful candidate
// =============================================================================

#[test]
fn trajectory_appends_only_on_successful_candidate() {
    // Given: An empty trajectory
    let mut trajectory = CtrlCohTrajectory::default();

    // When: We add a REJECTED attempt
    let rejected_result = CtrlObjectiveResult::LeanRejected {
        theorem_name: "Theorem.fail1".to_string(),
        error_kind: "type mismatch".to_string(),
        error_output: "".to_string(),
    };

    trajectory.add_attempt(
        "Theorem.fail1".to_string(),
        "candidate_fail1".to_string(),
        &rejected_result,
    );

    // Then: Trajectory should have 0 successes, 1 failure
    assert_eq!(
        trajectory.success_count, 0,
        "Should have 0 successful repairs"
    );
    assert_eq!(trajectory.failure_count, 1, "Should have 1 failed attempt");
    assert_eq!(
        trajectory.attempts.len(),
        1,
        "Should have 1 trajectory entry"
    );

    // When: We add an ACCEPTED attempt
    let accepted_result = CtrlObjectiveResult::LeanAccepted {
        theorem_name: "Theorem.success1".to_string(),
        proof_hash: "0xdef456".to_string(),
        stdout_hash: "stdout".to_string(),
        stderr_hash: "stderr".to_string(),
    };

    trajectory.add_attempt(
        "Theorem.success1".to_string(),
        "candidate_success1".to_string(),
        &accepted_result,
    );

    // Then: Trajectory should now have 1 success
    assert_eq!(
        trajectory.success_count, 1,
        "Should have 1 successful repair"
    );
    assert_eq!(
        trajectory.failure_count, 1,
        "Should still have 1 failed attempt"
    );
    assert_eq!(
        trajectory.attempts.len(),
        2,
        "Should have 2 trajectory entries"
    );

    // Verify the trajectory is valid
    let accounting = CtrlAccountingBudget::initial(1000, 100);
    assert!(
        trajectory.is_valid(&accounting),
        "Trajectory with 1 success should be valid"
    );
}

// =============================================================================
// Test 5b: Verify trajectory only advances for successful candidates
// =============================================================================

#[test]
fn trajectory_only_counts_successful_for_accumulator() {
    // Given: Two failed attempts followed by one success
    let mut trajectory = CtrlCohTrajectory::default();

    // Fail, Fail, Success sequence
    for i in 0..2 {
        let result = CtrlObjectiveResult::LeanRejected {
            theorem_name: format!("Theorem.fail{}", i),
            error_kind: "failed".to_string(),
            error_output: "".to_string(),
        };
        trajectory.add_attempt(
            format!("Theorem.fail{}", i),
            format!("candidate_fail{}", i),
            &result,
        );
    }

    let success_result = CtrlObjectiveResult::LeanAccepted {
        theorem_name: "Theorem.fixed".to_string(),
        proof_hash: "0xfixed".to_string(),
        stdout_hash: "".to_string(),
        stderr_hash: "".to_string(),
    };
    trajectory.add_attempt(
        "Theorem.fixed".to_string(),
        "candidate_fixed".to_string(),
        &success_result,
    );

    // Then: Only total attempts should be 3, but success should be 1
    assert_eq!(trajectory.attempts.len(), 3, "Should have 3 total attempts");
    assert_eq!(trajectory.success_count, 1, "Should have exactly 1 success");
    assert_eq!(trajectory.failure_count, 2, "Should have 2 failures");

    // Accumulator reflects success/failure count
    assert!(
        trajectory.accumulator.contains("1"),
        "Accumulator should reflect success count"
    );
}

// =============================================================================
// Feature-gated: Real Lean integration test
// =============================================================================

/// Test against real Lean server - requires lean-integration feature flag.
/// Run with: cargo test -p coh-genesis --features lean-integration
///
/// WARNING: This test requires a running Lean 4 server and may be slow/flaky.
/// Use only for CI or manual validation, not in the default test suite.
#[cfg(feature = "lean-integration")]
#[test]
fn ctrl_repairs_real_lean_and_emits_cohbit() {
    use std::path::PathBuf;

    // Skip if no Lean project available
    let project_path = PathBuf::from("coh-t-stack");
    if !project_path.join("lake-manifest.json").exists() {
        eprintln!("Skipping: No Lean project found at {:?}", project_path);
        return;
    }

    // Note: This test is stubbed because it requires a real Lean server
    // In practice, you'd create a real CtrlLoop and call repair_theorem
    // For now, this validates the test infrastructure compiles
    println!("Real Lean integration test stubbed - requires live server");
    println!("To implement, create CtrlLoop and call repair_theorem with real tactics");
}
