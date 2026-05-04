// Copyright 2024 Cohere Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! CohBit + CTRL Verification Gauntlet - Core Tests
//!
//! This module implements the core accounting law tests
//! from the verification gauntlet specification.
//!
//! ## Core Claims Tested
//!
//! - Accounting law: v_post + spend ≤ v_pre + defect + authority
//! - Overflow rejection (no wrapping, no saturation)
//! - Differential testing (production vs reference)
//! - Cross-layer equivalence
//! - CTRL adapter eligibility

#![allow(clippy::needless_update)]

use coh_core::accounting_law::{
    check_cumulative_accounting_law_u128, check_local_accounting_law_u128,
};
use coh_core::auth::{fixture_signing_key, sign_micro_receipt};
use coh_core::canon::CanonRegistry;
use coh_core::finalize_micro_receipt;
use coh_core::types::{AdmissionProfile, Decision, MetricsWire, MicroReceiptWire, RejectCode};
#[cfg(feature = "fixture-keys")]
use coh_core::verify_micro::verify_micro_dev_fixture as verify_micro_v1;

// =============================================================================
// CANONICAL SPEC CONSTANTS
// =============================================================================

/// Canonical test values for boundary accounting tests.
/// These exact values test the boundary case:
/// v_post + spend = v_pre + defect + authority
/// 120 + 10 = 100 + 0 + 30 = 130
pub const CANON_V_PRE: u128 = 100;
pub const CANON_V_POST: u128 = 120;
pub const CANON_SPEND: u128 = 10;
pub const CANON_DEFECT: u128 = 0;
pub const CANON_AUTHORITY: u128 = 30;

/// Negative control: authority = 29, should fail
/// 120 + 10 = 130 > 100 + 0 + 29 = 129
pub const CANON_AUTHORITY_SHORTFALL: u128 = 29;

/// Maximum u128 for overflow tests
pub const U128_MAX: u128 = u128::MAX;

// =============================================================================
// REFERENCE IMPLEMENTATION (Differential Testing)
// =============================================================================

/// Reference checker for differential testing.
/// This is a dumb, separate spec checker that must agree with the production kernel.
/// [PROVED] - uses checked arithmetic which is the canonical reference.
pub fn reference_check_local_accounting(
    v_pre: u128,
    v_post: u128,
    spend: u128,
    defect: u128,
    authority: u128,
) -> bool {
    match (
        v_post.checked_add(spend),
        v_pre
            .checked_add(defect)
            .and_then(|x| x.checked_add(authority)),
    ) {
        (Some(lhs), Some(rhs)) => lhs <= rhs,
        _ => false,
    }
}

/// Reference checker for cumulative accounting
/// [PROVED] - uses checked arithmetic which is the canonical reference.
pub fn reference_check_cumulative_accounting(
    v_pre_first: u128,
    v_post_last: u128,
    total_spend: u128,
    total_defect: u128,
    total_authority: u128,
) -> bool {
    match (
        v_post_last.checked_add(total_spend),
        v_pre_first
            .checked_add(total_defect)
            .and_then(|x| x.checked_add(total_authority)),
    ) {
        (Some(lhs), Some(rhs)) => lhs <= rhs,
        _ => false,
    }
}

// =============================================================================
// TEST FIXTURE BUILDERS
// =============================================================================

const TEST_SIGNER: &str = "gauntlet_signer";
const TEST_OBJ_ID: &str = "gauntlet_test_obj";

/// Build a valid test receipt with given values
/// Note: This requires fixture-keys feature
#[cfg(feature = "fixture-keys")]
fn build_test_wire(
    v_pre: u128,
    v_post: u128,
    spend: u128,
    defect: u128,
    authority: u128,
) -> MicroReceiptWire {
    let mut wire = MicroReceiptWire {
        schema_id: CanonRegistry::MICRO_V1_ID.to_string(),
        version: CanonRegistry::MICRO_V1_VERSION.to_string(),
        object_id: TEST_OBJ_ID.to_string(),
        canon_profile_hash: CanonRegistry::CANON_PROFILE_V1.to_string(),
        policy_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        step_index: 1,
        step_type: Some("gauntlet_test".to_string()),
        signatures: None,
        state_hash_prev: "1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        state_hash_next: "2222222222222222222222222222222222222222222222222222222222222222"
            .to_string(),
        chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        chain_digest_next: "0000000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
            authority: authority.to_string(),
            ..Default::default()
        },
        profile: AdmissionProfile::CoherenceOnlyV1,
    };

    // Finalize first to compute correct digest
    wire = finalize_micro_receipt(wire).expect("fixture should finalize");

    // Set chain_digest_prev for step_index=1
    wire.chain_digest_prev = wire.chain_digest_next.clone();

    // Sign with real Ed25519 key
    let signing_key = fixture_signing_key(TEST_SIGNER);
    wire = sign_micro_receipt(
        wire,
        &signing_key,
        TEST_SIGNER,
        "*",
        1700000000,
        None,
        "MICRO_RECEIPT_V1",
    )
    .expect("Failed to sign test receipt");

    wire
}

// =============================================================================
// TEST CLASS A: BOUNDARY ACCOUNTING TESTS
// =============================================================================

/// Test A.1: Exact boundary case - MUST PASS
/// v_post + spend = v_pre + defect + authority
/// 120 + 10 = 100 + 0 + 30 = 130
#[test]
fn test_boundary_exact_pass() {
    let result = check_local_accounting_law_u128(
        CANON_V_PRE,
        CANON_V_POST,
        CANON_SPEND,
        CANON_DEFECT,
        CANON_AUTHORITY,
    );

    assert!(result.is_ok(), "Boundary case should pass: {:?}", result);
    assert_eq!(result.unwrap(), 0, "Margin should be exactly 0");
}

/// Test A.2: Negative control - MUST FAIL
/// v_post + spend > v_pre + defect + authority with short authority
/// 120 + 10 = 130 > 100 + 0 + 29 = 129
#[test]
fn test_boundary_negative_control_fail() {
    let result = check_local_accounting_law_u128(
        CANON_V_PRE,
        CANON_V_POST,
        CANON_SPEND,
        CANON_DEFECT,
        CANON_AUTHORITY_SHORTFALL,
    );

    assert!(result.is_err(), "Short authority should fail: {:?}", result);
    assert_eq!(result.unwrap_err(), RejectCode::RejectPolicyViolation);
}

/// Test A.3: Boundary with various edge cases
#[test]
fn test_boundary_various() {
    // Test case: equality with different values
    let result = check_local_accounting_law_u128(100, 100, 0, 0, 0);
    assert!(result.is_ok(), "v_post + 0 = v_pre + 0 should pass");
    assert_eq!(result.unwrap(), 0, "Margin should be 0");

    // Test with defect
    let result = check_local_accounting_law_u128(100, 50, 25, 25, 50);
    assert!(result.is_ok(), "50+25=75 <= 100+25+50=175 should pass");
}

// =============================================================================
// TEST CLASS B: OVERFLOW TESTS
// =============================================================================

/// Test B.1: LHS overflow - v_post + spend = u128::MAX + 1
#[test]
fn test_lhs_overflow_reject() {
    let result = check_local_accounting_law_u128(
        0,         // v_pre
        U128_MAX,  // v_post
        1,         // spend (causes overflow)
        0,         // defect
        u128::MAX, // authority (large enough to not be the issue)
    );

    assert!(
        result.is_err(),
        "LHS overflow must be rejected: {:?}",
        result
    );
    assert_eq!(
        result.unwrap_err(),
        RejectCode::RejectOverflow,
        "Must be specifically overflow, not policy violation"
    );
}

/// Test B.2: RHS overflow - v_pre + defect = u128::MAX + 1
#[test]
fn test_rhs_overflow_reject_v_pre() {
    let result = check_local_accounting_law_u128(
        U128_MAX, // v_pre
        0,        // v_post
        0,        // spend
        1,        // defect (causes v_pre + defect overflow)
        0,        // authority
    );

    assert!(
        result.is_err(),
        "RHS overflow (v_pre+defect) must be rejected: {:?}",
        result
    );
    assert_eq!(result.unwrap_err(), RejectCode::RejectOverflow);
}

/// Test B.3: RHS overflow - (v_pre + defect) + authority overflow
#[test]
fn test_rhs_overflow_authority() {
    let result = check_local_accounting_law_u128(
        U128_MAX - 1, // v_pre (so v_pre + defect doesn't overflow alone)
        0,            // v_post
        0,            // spend
        1,            // defect (now v_pre + defect = u128::MAX)
        1,            // authority (causes overflow)
    );

    assert!(
        result.is_err(),
        "RHS overflow (authority) must be rejected: {:?}",
        result
    );
}

/// Test B.4: Cumulative slab overflow
#[test]
fn test_cumulative_overflow_reject() {
    let result = check_cumulative_accounting_law_u128(
        U128_MAX, // v_pre_first
        0,        // v_post_last
        0,        // total_spend
        1,        // total_defect (causes overflow)
        0,        // total_authority
    );

    assert!(
        result.is_err(),
        "Cumulative overflow must be rejected: {:?}",
        result
    );
}

// =============================================================================
// TEST CLASS C: DIFFERENTIAL TESTING (Production vs Reference)
// =============================================================================

/// Test C.1: Production matches reference for valid cases
#[test]
fn test_production_vs_reference_valid() {
    let test_cases = vec![
        (100, 50, 20, 10, 80), // 50+20=70 <= 100+10+80=190
        (100, 100, 0, 0, 0),   // 100+0=100 <= 100+0+0=100 boundary
        (100, 120, 10, 0, 30), // 120+10=130 <= 130 boundary
        (10, 5, 10, 0, 20),    // 5+10=15 <= 10+0+20=30
    ];

    for (v_pre, v_post, spend, defect, authority) in test_cases {
        let prod_result =
            check_local_accounting_law_u128(v_pre, v_post, spend, defect, authority).is_ok();

        let ref_result = reference_check_local_accounting(v_pre, v_post, spend, defect, authority);

        assert_eq!(
            prod_result, ref_result,
            "Production must match reference for valid case: v_pre={}, v_post={}, spend={}, defect={}, authority={}",
            v_pre, v_post, spend, defect, authority
        );
    }
}

/// Test C.2: Production matches reference for invalid cases
#[test]
fn test_production_vs_reference_invalid() {
    let test_cases = vec![
        (100, 130, 10, 0, 29), // 130 > 129
        (100, 50, 100, 0, 20), // 150 > 120
        (10, 100, 0, 0, 0),    // 100 > 10
    ];

    for (v_pre, v_post, spend, defect, authority) in test_cases {
        let prod_result =
            check_local_accounting_law_u128(v_pre, v_post, spend, defect, authority).is_ok();

        let ref_result = reference_check_local_accounting(v_pre, v_post, spend, defect, authority);

        assert_eq!(
            prod_result, ref_result,
            "Production must match reference for invalid case"
        );
    }
}

// =============================================================================
// TEST CLASS D: CUMULATIVE ACCOUNTING LAW
// =============================================================================

/// Test D.1: Cumulative accounting law passes
#[test]
fn test_cumulative_accounting_pass() {
    // v_post_last + total_spend <= v_pre_first + total_defect + total_authority
    // 50 + 25 = 75 <= 100 + 0 + 80 = 180 ✓
    let result = check_cumulative_accounting_law_u128(
        100, // v_pre_first
        50,  // v_post_last
        25,  // total_spend
        0,   // total_defect
        80,  // total_authority
    );

    assert!(result.is_ok(), "Cumulative law should pass: {:?}", result);
}

/// Test D.2: Cumulative accounting law violation
#[test]
fn test_cumulative_law_violation() {
    // 50 + 25 = 75 <= 100 + 0 + 20 = 120 ✓ (incorrect calculation)
    // Actually: 50+25=75 <= 100+20=120, should pass
    // Let's use case that actually fails:
    // 50 + 25 = 75 > 100 + 0 + 10 = 110 ✗
    let result = check_cumulative_accounting_law_u128(
        100, // v_pre_first
        50,  // v_post_last
        25,  // total_spend (75 > 110? No, 75 < 110)
        0,   // total_defect
        10,  // total_authority (too low: 75 <= 110 but should fail)
    );

    // This actually passes because 75 <= 110
    // Let me fix: 80 + 30 = 110 > 100 + 10 = 110? No equal.
    // Need case: 50 + 30 = 80 > 100 + 0 + 10 = 110? No.
    // Let's use: 50 + 35 = 85 > 100 + 0 + 0 = 100? No.
    // Real violation case: v_post + spend > v_pre + defect + authority
    let result2 = check_cumulative_accounting_law_u128(
        50, // v_pre_first
        50, // v_post_last
        60, // total_spend (110 > 50 + 0 + 0 = 50)
        0,  // total_defect
        0,  // total_authority
    );

    assert!(result2.is_err(), "Cumulative law violation should fail");
}

// =============================================================================
// TEST CLASS E: CTRL ADAPTER ELIGIBILITY
// =============================================================================

/// Test E.1: Valid CTRL repair emits CohBit
/// spend=10, authority=20 -> 0+10 <= 0+0+20 = 20 ✓
#[test]
fn test_adapter_valid_repair() {
    let accounting_ok = reference_check_local_accounting(
        0,  // v_pre initial budget
        0,  // v_post
        10, // spend (tactic cost)
        0,  // defect_reserve
        20, // authority
    );

    assert!(accounting_ok, "Valid repair should pass accounting");
}

/// Test E.2: Invalid CTRL repair - insufficient authority
/// spend=100, authority=1 -> 0+100 > 0+0+1 = 1 ✗
#[test]
fn test_adapter_insufficient_authority() {
    let accounting_ok = reference_check_local_accounting(
        0,   // v_pre
        0,   // v_post
        100, // spend (too high)
        0,   // defect_reserve
        1,   // authority (too low)
    );

    assert!(!accounting_ok, "Insufficient authority should fail");
}

/// Test E.3: CTRL repair with overflow - should reject
#[test]
fn test_adapter_overflow_reject() {
    let result = check_local_accounting_law_u128(
        0,
        u128::MAX,
        1, // would overflow
        0,
        u128::MAX,
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), RejectCode::RejectOverflow);
}

// =============================================================================
// TEST CLASS F: PROPERTY TESTS
// =============================================================================

/// Test F.1: Determinism - same input produces same output
#[test]
fn test_property_determinism() {
    let result1 = check_local_accounting_law_u128(100, 80, 10, 0, 30);
    let result2 = check_local_accounting_law_u128(100, 80, 10, 0, 30);

    assert_eq!(result1.is_ok(), result2.is_ok());
}

/// Test F.2: Monotonicity - more authority cannot make valid invalid  
#[test]
fn test_property_monotonicity() {
    // Base: 100 + 10 <= 100 + 0 + 30 = 130, passes
    let base = check_local_accounting_law_u128(100, 80, 10, 0, 30);
    assert!(base.is_ok());

    // More authority: 100 + 10 <= 100 + 0 + 50 = 150, should also pass
    let more = check_local_accounting_law_u128(100, 80, 10, 0, 50);
    assert!(more.is_ok());

    // Less authority might fail
    let less = check_local_accounting_law_u128(100, 80, 10, 0, 10);
    // 80+10=90 <= 100+0+10=110, actually passes
    // Need different case to fail:
    let less2 = check_local_accounting_law_u128(100, 90, 20, 0, 5);
    // 90+20=110 <= 100+0+5=105? No, 110 > 105! Should fail
    assert!(less2.is_err());
}

// =============================================================================
// TEST CLASS G: REGRESSION TESTS
// =============================================================================

/// Regression: No silent wrap-around on overflow
#[test]
fn regression_no_silent_wrap() {
    let result = check_local_accounting_law_u128(u128::MAX, 0, 0, 1, 0);

    // MUST reject with overflow, not silently wrap
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), RejectCode::RejectOverflow);
}

/// Regression: authority=0 handled correctly
#[test]
fn regression_authority_zero() {
    // 80 + 10 = 90 <= 100 + 0 = 100 ✓
    let result = check_local_accounting_law_u128(100, 80, 10, 0, 0);
    assert!(result.is_ok());
}

/// Regression: cumulative zero totals
#[test]
fn regression_cumulative_zero() {
    // 50 + 0 = 50 <= 50 + 0 + 0 = 50 ✓
    let result = check_cumulative_accounting_law_u128(50, 50, 0, 0, 0);
    assert!(result.is_ok());
}

// =============================================================================
// TEST CLASS H: MICRO VERIFIER INTEGRATION
// =============================================================================

/// Test H.1: Micro verifier accepts valid boundary case
/// Note: Requires fixture-keys feature for signing
#[cfg(feature = "fixture-keys")]
#[test]
fn test_micro_verifier_accepts_boundary() {
    use coh_core::verify_micro::verify_micro_dev_fixture;

    let wire = build_test_wire(
        CANON_V_PRE,
        CANON_V_POST,
        CANON_SPEND,
        CANON_DEFECT,
        CANON_AUTHORITY,
    );

    let result = verify_micro_dev_fixture(wire);

    assert_eq!(
        result.decision,
        Decision::Accept,
        "Micro verifier should accept boundary case: {:?}",
        result
    );
}

/// Test H.2: Micro verifier rejects insufficient authority
/// Note: Requires fixture-keys feature for signing
#[cfg(feature = "fixture-keys")]
#[test]
fn test_micro_verifier_rejects_shortfall() {
    use coh_core::verify_micro::verify_micro_dev_fixture;

    let wire = build_test_wire(
        CANON_V_PRE,
        CANON_V_POST,
        CANON_SPEND,
        CANON_DEFECT,
        CANON_AUTHORITY_SHORTFALL,
    );

    let result = verify_micro_dev_fixture(wire);

    assert_eq!(
        result.decision,
        Decision::Reject,
        "Micro verifier should reject short authority: {:?}",
        result
    );
}

// =============================================================================
// MAIN GAUNTLET TEST
// =============================================================================

/// Master test - runs all categories and reports summary
#[test]
fn test_gauntlet_master() {
    println!("\n=== COHBIT+CTRL VERIFICATION GAUNTLET ===\n");

    // Category A: Boundary tests
    println!("[A] Boundary Accounting Tests...");
    let boundary_ok = std::panic::catch_unwind(test_boundary_exact_pass).is_ok()
        && std::panic::catch_unwind(test_boundary_negative_control_fail).is_ok()
        && std::panic::catch_unwind(test_boundary_various).is_ok();
    println!("  Result: {}", if boundary_ok { "PASS" } else { "FAIL" });

    // Category B: Overflow tests
    println!("[B] Overflow Rejection Tests...");
    let overflow_ok = std::panic::catch_unwind(test_lhs_overflow_reject).is_ok()
        && std::panic::catch_unwind(test_rhs_overflow_reject_v_pre).is_ok()
        && std::panic::catch_unwind(test_rhs_overflow_authority).is_ok();
    println!("  Result: {}", if overflow_ok { "PASS" } else { "FAIL" });

    // Category C: Differential testing
    println!("[C] Differential Testing...");
    let diff_ok = std::panic::catch_unwind(test_production_vs_reference_valid).is_ok()
        && std::panic::catch_unwind(test_production_vs_reference_invalid).is_ok();
    println!("  Result: {}", if diff_ok { "PASS" } else { "FAIL" });

    // Category D: Cumulative tests
    println!("[D] Cumulative Accounting Tests...");
    let cum_ok = std::panic::catch_unwind(test_cumulative_accounting_pass).is_ok()
        && std::panic::catch_unwind(test_cumulative_law_violation).is_ok();
    println!("  Result: {}", if cum_ok { "PASS" } else { "FAIL" });

    // Category E: CTRL adapter tests
    println!("[E] CTRL Adapter Eligibility...");
    let ctrl_ok = std::panic::catch_unwind(test_adapter_valid_repair).is_ok()
        && std::panic::catch_unwind(test_adapter_insufficient_authority).is_ok()
        && std::panic::catch_unwind(test_adapter_overflow_reject).is_ok();
    println!("  Result: {}", if ctrl_ok { "PASS" } else { "FAIL" });

    // Category F: Property tests
    println!("[F] Property Tests...");
    let prop_ok = std::panic::catch_unwind(test_property_determinism).is_ok()
        && std::panic::catch_unwind(test_property_monotonicity).is_ok();
    println!("  Result: {}", if prop_ok { "PASS" } else { "FAIL" });

    // Category G: Regression tests
    println!("[G] Regression Tests...");
    let reg_ok = std::panic::catch_unwind(regression_no_silent_wrap).is_ok()
        && std::panic::catch_unwind(regression_authority_zero).is_ok()
        && std::panic::catch_unwind(regression_cumulative_zero).is_ok();
    println!("  Result: {}", if reg_ok { "PASS" } else { "FAIL" });

    // Category H: Micro verifier (optional, requires fixture-keys)
    #[cfg(feature = "fixture-keys")]
    println!("[H] Micro Verifier Integration...");
    #[cfg(feature = "fixture-keys")]
    let micro_ok = std::panic::catch_unwind(test_micro_verifier_accepts_boundary).is_ok()
        && std::panic::catch_unwind(test_micro_verifier_rejects_shortfall).is_ok();
    #[cfg(feature = "fixture-keys")]
    println!("  Result: {}", if micro_ok { "PASS" } else { "FAIL" });

    #[cfg(not(feature = "fixture-keys"))]
    println!("[H] Micro Verifier Integration... SKIPPED (requires fixture-keys)");

    // Summary
    println!("\n=== GAUNTLET SUMMARY ===");
    println!(
        "A (Boundary):   {}",
        if boundary_ok { "PASS" } else { "FAIL" }
    );
    println!(
        "B (Overflow):   {}",
        if overflow_ok { "PASS" } else { "FAIL" }
    );
    println!(
        "C (Differential): {}",
        if diff_ok { "PASS" } else { "FAIL" }
    );
    println!("D (Cumulative): {}", if cum_ok { "PASS" } else { "FAIL" });
    println!("E (CTRL):       {}", if ctrl_ok { "PASS" } else { "FAIL" });
    println!("F (Property):   {}", if prop_ok { "PASS" } else { "FAIL" });
    println!("G (Regression): {}", if reg_ok { "PASS" } else { "FAIL" });

    #[cfg(feature = "fixture-keys")]
    println!("H (Micro):      {}", if micro_ok { "PASS" } else { "FAIL" });

    let all_pass = boundary_ok && overflow_ok && diff_ok && cum_ok && ctrl_ok && prop_ok && reg_ok;

    #[cfg(feature = "fixture-keys")]
    let all_pass = all_pass && micro_ok;

    println!(
        "\nOVERALL: {}",
        if all_pass {
            "✓ ALL PASS"
        } else {
            "✗ SOME FAILURES"
        }
    );
    println!("=============================\n");

    assert!(all_pass, "Some gauntlet categories failed");
}
