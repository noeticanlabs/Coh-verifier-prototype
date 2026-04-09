use crate::types::{MicroReceiptWire, MicroReceipt, VerifyMicroResult, Decision, RejectCode};
use crate::math::CheckedMath;
use crate::canon::{to_prehash_view, to_canonical_json_bytes, EXPECTED_MICRO_SCHEMA_ID, EXPECTED_MICRO_VERSION, EXPECTED_CANON_PROFILE_HASH};
use crate::hash::compute_chain_digest;
use std::convert::TryFrom;

pub fn verify_micro(wire: MicroReceiptWire) -> VerifyMicroResult {
    // 1. Wire to runtime conversion
    let r = match MicroReceipt::try_from(wire) {
        Ok(r) => r,
        Err(e) => return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(e.clone()), 
            message: format!("Wire conversion failed: {:?}", e),
            step_index: None,
            object_id: None,
            chain_digest_next: None,
        },
    };

    // 2. Schema check
    if r.schema_id != EXPECTED_MICRO_SCHEMA_ID {
        return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectSchema),
            message: format!("Invalid schema_id: {}", r.schema_id),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }
    if r.version != EXPECTED_MICRO_VERSION {
        return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectSchema),
            message: format!("Unsupported version: {}", r.version),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 3. Canon profile check
    if r.canon_profile_hash.to_hex() != EXPECTED_CANON_PROFILE_HASH {
        return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectCanonProfile),
            message: "Canon profile hash mismatch".to_string(),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 4. Field sanity
    if r.object_id.is_empty() {
        return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectSchema),
            message: "object_id is empty".to_string(),
            step_index: Some(r.step_index),
            object_id: None,
            chain_digest_next: None,
        };
    }

    // 5. Checked arithmetic
    // v_post + spend
    let left_side = match r.metrics.v_post.safe_add(r.metrics.spend) {
        Ok(val) => val,
        Err(e) => return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(e.clone()),
            message: format!("Arithmetic overflow on left side: {:?}", e),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        },
    };
    // v_pre + defect
    let right_side = match r.metrics.v_pre.safe_add(r.metrics.defect) {
        Ok(val) => val,
        Err(e) => return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(e.clone()),
            message: format!("Arithmetic overflow on right side: {:?}", e),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        },
    };

    // 6. Policy inequality
    if left_side > right_side {
        return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectPolicyViolation),
            message: format!("v_post + spend ({}) exceeds v_pre + defect ({})", left_side, right_side),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 7. Digest recomputation
    let prehash = to_prehash_view(&r);
    let canon_bytes = match to_canonical_json_bytes(&prehash) {
        Ok(b) => b,
        Err(_) => return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectNumericParse),
            message: "Canonical serialization failed".to_string(),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        },
    };
    let recomputed_digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);

    // 8. Digest compare
    if r.chain_digest_next != recomputed_digest {
        return VerifyMicroResult { 
            decision: Decision::Reject, 
            code: Some(RejectCode::RejectChainDigest),
            message: format!("Digest mismatch. Received: {}, Computed: {}", r.chain_digest_next.to_hex(), recomputed_digest.to_hex()),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: Some(r.chain_digest_next.to_hex()),
        };
    }

    VerifyMicroResult { 
        decision: Decision::Accept, 
        code: None,
        message: "Micro-receipt accepted".to_string(),
        step_index: Some(r.step_index),
        object_id: Some(r.object_id),
        chain_digest_next: Some(r.chain_digest_next.to_hex()),
    }
}
