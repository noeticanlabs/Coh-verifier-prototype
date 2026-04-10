use crate::types::{SlabReceiptWire, SlabReceipt, VerifySlabResult, Decision, RejectCode};
use crate::math::CheckedMath;
use crate::canon::{EXPECTED_SLAB_SCHEMA_ID, EXPECTED_SLAB_VERSION};
use std::convert::TryFrom;

pub fn verify_slab(wire: SlabReceiptWire) -> VerifySlabResult {
    // 1. Wire to runtime
    let r = match SlabReceipt::try_from(wire) {
        Ok(r) => r,
        Err(e) => return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(e.clone()), 
            message: format!("Wire conversion failed: {:?}", e),
            range_start: 0,
            range_end: 0,
            micro_count: None,
            merkle_root: None,
        },
    };

    // 2. Envelope checks
    if r.schema_id != EXPECTED_SLAB_SCHEMA_ID {
        return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectSchema), 
            message: format!("Invalid schema_id: {} (Expected: {})", r.schema_id, EXPECTED_SLAB_SCHEMA_ID),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }
    if r.version != EXPECTED_SLAB_VERSION {
        return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectSchema), 
            message: format!("Unsupported version: {} (Expected: {})", r.version, EXPECTED_SLAB_VERSION),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    // 3. Range sanity (Slab Summary layer)
    if r.micro_count == 0 {
        return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectSlabSummary), 
            message: "Slab is empty (micro_count = 0). Slab must contain at least one micro-receipt.".to_string(),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(0),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }
    if r.range_end < r.range_start {
        return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectSlabSummary), 
            message: format!("Invalid range: {}..{} (End index cannot be less than start index)", r.range_start, r.range_end),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }
    
    // Exactly count matches interval check:
    let expected_count = r.range_end - r.range_start + 1;
    if expected_count != r.micro_count {
        return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectSlabSummary), 
            message: format!("Range count mismatch: interval {}..{} implies {} steps, but micro_count is {}", r.range_start, r.range_end, expected_count, r.micro_count),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }
    
    // 4. Macro inequality
    let left_side = match r.summary.v_post_last.safe_add(r.summary.total_spend) {
        Ok(val) => val,
        Err(e) => return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(e.clone()), 
            message: format!("Macro arithmetic overflow (v_post_last + total_spend): {:?}", e),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        },
    };
    let right_side = match r.summary.v_pre_first.safe_add(r.summary.total_defect) {
        Ok(val) => val,
        Err(e) => return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(e.clone()), 
            message: format!("Macro arithmetic overflow (v_pre_first + total_defect): {:?}", e),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        },
    };

    if left_side > right_side {
        return VerifySlabResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectPolicyViolation), 
            message: format!("Macro inequality violated: v_post_last + total_spend ({}) exceeds v_pre_first + total_defect ({})", left_side, right_side),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    VerifySlabResult { 
        decision: Decision::Accept, 
        code: None, 
        message: "Slab verified successfully: range checked and macro-accounting balanced.".to_string(),
        range_start: r.range_start,
        range_end: r.range_end,
        micro_count: Some(r.micro_count),
        merkle_root: Some(r.merkle_root.to_hex()),
    }
}
