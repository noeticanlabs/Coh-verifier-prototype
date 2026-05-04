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

use crate::reject::RejectCode;

/// Result of accounting law check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountingLawResult {
    /// Law is satisfied
    Satisfied,
    /// Law is violated
    Violated,
}

/// Check the canonical local accounting law using u128.
///
/// Local inequality: v_post + spend ≤ v_pre + defect + authority
///
/// This is the SINGLE SOURCE OF TRUTH for admissibility.
/// All verifier paths (L0, V3, slab, chain, dashboard) must use this.
pub fn check_local_accounting_law_u128(
    v_pre: u128,
    v_post: u128,
    spend: u128,
    defect: u128,
    authority: u128,
) -> Result<u128, RejectCode> {
    // LHS: v_post + spend
    let lhs = match v_post.checked_add(spend) {
        Some(v) => v,
        None => return Err(RejectCode::RejectOverflow),
    };

    // RHS: v_pre + defect + authority
    let rhs = match v_pre.checked_add(defect) {
        Some(v) => match v.checked_add(authority) {
            Some(r) => r,
            None => return Err(RejectCode::RejectOverflow),
        },
        None => return Err(RejectCode::RejectOverflow),
    };

    // Check: lhs ≤ rhs
    if lhs <= rhs {
        Ok(rhs - lhs) // margin (surplus)
    } else {
        Err(RejectCode::RejectPolicyViolation)
    }
}

/// Check cumulative accounting law for slabs.
///
/// Cumulative inequality: v_post_last + total_spend ≤ v_pre_first + total_defect + total_authority
pub fn check_cumulative_accounting_law_u128(
    v_pre_first: u128,
    v_post_last: u128,
    total_spend: u128,
    total_defect: u128,
    total_authority: u128,
) -> Result<u128, RejectCode> {
    // LHS: v_post_last + total_spend
    let lhs = match v_post_last.checked_add(total_spend) {
        Some(v) => v,
        None => return Err(RejectCode::RejectOverflow),
    };

    // RHS: v_pre_first + total_defect + total_authority
    let rhs = match v_pre_first.checked_add(total_defect) {
        Some(v) => match v.checked_add(total_authority) {
            Some(r) => r,
            None => return Err(RejectCode::RejectOverflow),
        },
        None => return Err(RejectCode::RejectOverflow),
    };

    // Check: lhs ≤ rhs
    if lhs <= rhs {
        Ok(rhs - lhs) // margin
    } else {
        Err(RejectCode::RejectPolicyViolation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_law_satisfied() {
        // v_post + spend = 120 + 10 = 130
        // v_pre + defect + authority = 100 + 0 + 30 = 130
        // 130 ≤ 130: satisfied
        let result = check_local_accounting_law_u128(100, 120, 10, 0, 30);
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_law_violated() {
        // v_post + spend = 130 + 10 = 140
        // v_pre + defect + authority = 100 + 0 + 30 = 130
        // 140 > 130: violated
        let result = check_local_accounting_law_u128(100, 130, 10, 0, 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_cumulative_law_satisfied() {
        let result = check_cumulative_accounting_law_u128(100, 120, 10, 0, 30);
        assert!(result.is_ok());
    }

    #[test]
    fn test_overflow_rejected() {
        // u128::MAX + 1 would overflow
        let v_post = u128::MAX;
        let spend = 1u128;
        let result = check_local_accounting_law_u128(100, v_post, spend, 0, 30);
        assert_eq!(result, Err(RejectCode::RejectOverflow));
    }
}
