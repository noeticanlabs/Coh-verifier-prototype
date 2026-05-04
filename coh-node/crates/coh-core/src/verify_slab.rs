use crate::accounting_law::check_cumulative_accounting_law_u128;
use crate::canon::CanonRegistry;
use crate::merkle;
use crate::types::{Decision, RejectCode, SlabReceipt, SlabReceiptWire, VerifySlabResult};
use std::convert::TryFrom;

/// NOTE: This verifies macro-accounting integrity but does NOT verify the Merkle root.
/// Full Merkle verification requires `verify_slab_with_leaves()`.
/// - `verify_slab_envelope()` = summary/envelope verification only
/// - `verify_slab_with_leaves()` = full merkle verification
#[must_use]
pub fn verify_slab_envelope(wire: SlabReceiptWire) -> VerifySlabResult {
    let r = match SlabReceipt::try_from(wire) {
        Ok(r) => r,
        Err(e) => {
            return VerifySlabResult {
                decision: Decision::Reject,
                code: Some(e),
                message: format!("Wire conversion failed: {:?}", e),
                range_start: 0,
                range_end: 0,
                micro_count: None,
                merkle_root: None,
            }
        }
    };

    if !CanonRegistry::validate_slab(&r.schema_id, &r.version) {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: format!(
                "Invalid schema_id/version: {} v{} (Expected: {} v{})",
                r.schema_id,
                r.version,
                CanonRegistry::SLAB_V1_ID,
                CanonRegistry::SLAB_V1_VERSION
            ),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    if r.micro_count == 0 {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabSummary),
            message: "Slab is empty (micro_count = 0).".to_string(),
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
            message: "Invalid range.".to_string(),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    let expected_count = r.range_end - r.range_start + 1;
    if expected_count != r.micro_count {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabSummary),
            message: "Range count mismatch.".to_string(),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    // Cumulative accounting law check for slab
    // Constraint: v_post_last + total_spend <= v_pre_first + total_defect + total_authority
    // Use shared accounting law kernel to prevent formula drift
    match check_cumulative_accounting_law_u128(
        r.summary.v_pre_first,
        r.summary.v_post_last,
        r.summary.total_spend,
        r.summary.total_defect,
        r.summary.authority,
    ) {
        Ok(_margin) => {
            // Accounting law satisfied
        }
        Err(code) => {
            return VerifySlabResult {
                decision: Decision::Reject,
                code: Some(code),
                message: "Macro accounting law check failed".to_string(),
                range_start: r.range_start,
                range_end: r.range_end,
                micro_count: Some(r.micro_count),
                merkle_root: Some(r.merkle_root.to_hex()),
            };
        }
    }

    VerifySlabResult {
        decision: Decision::Accept,
        code: None,
        message: "Slab verified successfully.".to_string(),
        range_start: r.range_start,
        range_end: r.range_end,
        micro_count: Some(r.micro_count),
        merkle_root: Some(r.merkle_root.to_hex()),
    }
}

#[must_use]
pub fn verify_slab_with_leaves(
    wire: SlabReceiptWire,
    leaves: Vec<crate::types::Hash32>,
) -> VerifySlabResult {
    let wire_clone = wire.clone();
    let mut result = verify_slab_envelope(wire);
    if result.decision != Decision::Accept {
        return result;
    }

    let slab = crate::types::SlabReceipt::try_from(wire_clone).unwrap();
    match merkle::verify_merkle_root(slab.merkle_root, &leaves) {
        Ok(()) => {
            result.message = "Slab verified successfully including Merkle root.".to_string();
            result
        }
        Err(()) => VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabMerkle),
            message: "Merkle root mismatch.".to_string(),
            range_start: slab.range_start,
            range_end: slab.range_end,
            micro_count: Some(slab.micro_count),
            merkle_root: Some(slab.merkle_root.to_hex()),
        },
    }
}
