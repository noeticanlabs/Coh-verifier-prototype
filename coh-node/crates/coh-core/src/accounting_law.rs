//! Shared Accounting Law Kernel
//!
//! One canonical function for the admissibility law. All verifier paths
//! must use this to ensure consistent semantics across the codebase.
//!
//! ## Canonical Local Law
//! v_post + spend ≤ v_pre + defect + authority
//!
//! ## Cumulative Law (Telescoping)  
//! ∑(v_post + spend - v_pre - defect - authority) ≤ 0

use crate::math::CheckedMath;
use crate::reject::RejectCode;
use crate::types::Rational64;

/// Result of accounting law check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountingLawResult {
    /// Law is satisfied, margin is surplus
    Satisfied(Rational64),
    /// Law is violated
    Violated,
}

/// Check the canonical local accounting law.
///
/// Local inequality: v_post + spend ≤ v_pre + defect + authority
///
/// This is the SINGLE SOURCE OF TRUTH for admissibility.
/// All verifier paths (L0, V3, slab, chain, dashboard) must use this.
pub fn check_local_accounting_law(
    v_pre: Rational64,
    v_post: Rational64,
    spend: Rational64,
    defect: Rational64,
    authority: Rational64,
) -> AccountingLawResult {
    let lhs = match v_post.safe_add(spend) {
        Ok(v) => v,
        Err(_) => return AccountingLawResult::Violated,
    };

    let rhs = match v_pre.safe_add(defect) {
        Ok(v) => match v.safe_add(authority) {
            Ok(r) => r,
            Err(_) => return AccountingLawResult::Violated,
        },
        Err(_) => return AccountingLawResult::Violated,
    };

    if lhs <= rhs {
        AccountingLawResult::Satisfied(rhs - lhs)
    } else {
        AccountingLawResult::Violated
    }
}

/// Check cumulative (trajectory) accounting law.
///
/// Cumulative inequality: ∑(v_post + spend - v_pre - defect - authority) ≤ 0
///
/// This telescopes from local law. Returns margin surplus if satisfied,
/// or negative if violated.
pub fn check_cumulative_accounting_law(
    v_pre_first: Rational64,
    v_post_last: Rational64,
    total_spend: Rational64,
    total_defect: Rational64,
    total_authority: Rational64,
) -> AccountingLawResult {
    let lhs = match v_post_last.safe_add(total_spend) {
        Ok(v) => v,
        Err(_) => return AccountingLawResult::Violated,
    };

    let rhs = match v_pre_first.safe_add(total_defect) {
        Ok(v) => match v.safe_add(total_authority) {
            Ok(r) => r,
            Err(_) => return AccountingLawResult::Violated,
        },
        Err(_) => return AccountingLawResult::Violated,
    };

    if lhs <= rhs {
        AccountingLawResult::Satisfied(rhs - lhs)
    } else {
        AccountingLawResult::Violated
    }
}

/// Variant with integer u128 for performance-critical paths
pub fn check_local_accounting_law_u128(
    v_pre: u128,
    v_post: u128,
    spend: u128,
    defect: u128,
    authority: u128,
) -> Result<u128, RejectCode> {
    let lhs = v_post
        .checked_add(spend)
        .ok_or(RejectCode::RejectOverflow)?;

    let rhs = v_pre
        .checked_add(defect)
        .ok_or(RejectCode::RejectOverflow)?
        .checked_add(authority)
        .ok_or(RejectCode::RejectOverflow)?;

    if lhs <= rhs {
        Ok(rhs - lhs) // margin surplus
    } else {
        Err(RejectCode::RejectPolicyViolation)
    }
}

/// Check cumulative with u128
pub fn check_cumulative_accounting_law_u128(
    v_pre_first: u128,
    v_post_last: u128,
    total_spend: u128,
    total_defect: u128,
    total_authority: u128,
) -> Result<u128, RejectCode> {
    let lhs = v_post_last
        .checked_add(total_spend)
        .ok_or(RejectCode::RejectOverflow)?;

    let rhs = v_pre_first
        .checked_add(total_defect)
        .ok_or(RejectCode::RejectOverflow)?
        .checked_add(total_authority)
        .ok_or(RejectCode::RejectOverflow)?;

    if lhs <= rhs {
        Ok(rhs - lhs)
    } else {
        Err(RejectCode::RejectPolicyViolation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_law_with_authority() {
        // v_post=120, spend=10, v_pre=100, defect=0, authority=30
        // 120+10=130 <= 100+0+30=130 ✓
        let result = check_local_accounting_law_u128(100, 120, 10, 0, 30);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // exactly at boundary
    }

    #[test]
    fn test_local_law_without_authority_fails() {
        // Same values but no authority would fail: 130 > 100
        let result = check_local_accounting_law_u128(100, 120, 10, 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_cumulative_telescoping() {
        // Two steps: (100→80: 80+5=85<=100+0) AND (80→60: 60+5=65<=80+0)
        // Total: (60+10=70) <= (100+0+0=100) ✓
        let result = check_cumulative_accounting_law_u128(100, 60, 10, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 40); // 100 - 60 - 10
    }

    #[test]
    fn test_cumulative_with_authority() {
        // With authority: v_post + spend <= v_pre + defect + authority
        let result = check_cumulative_accounting_law_u128(100, 60, 10, 0, 50);
        assert!(result.is_ok());
    }
}
