use crate::canon::CanonRegistry;
use crate::merkle::build_merkle_root;
use crate::types::{
    BuildSlabResult, Decision, MicroReceipt, MicroReceiptWire, RejectCode, SlabReceiptWire,
    SlabSummaryWire,
};
use crate::verify_chain::verify_chain;
use std::convert::TryFrom;

#[must_use]
pub fn build_slab(receipts: Vec<MicroReceiptWire>) -> BuildSlabResult {
    if receipts.is_empty() {
        return BuildSlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: "Empty chain provided".to_string(),
            range_start: None,
            range_end: None,
            micro_count: None,
            merkle_root: None,
            output: None,
            slab: None,
        };
    }

    // 1. Verify source chain
    let chain_res = verify_chain(receipts.clone());
    if chain_res.decision == Decision::Reject {
        return BuildSlabResult {
            decision: Decision::Reject,
            code: chain_res.code,
            message: format!("Source chain invalid: {}", chain_res.message),
            range_start: Some(chain_res.first_step_index),
            range_end: Some(chain_res.last_step_index),
            micro_count: Some(receipts.len() as u64),
            merkle_root: None,
            output: None,
            slab: None,
        };
    }

    // 2. Aggregate totals and collect leaves
    let mut total_spend: u128 = 0;
    let mut total_defect: u128 = 0;
    let mut total_delta: u128 = 0;
    let mut total_authority: u128 = 0; // Authority must be preserved in slab compression
    let first_wire = receipts.first().unwrap();
    let last_wire = receipts.last().unwrap();

    let mut leaves = Vec::with_capacity(receipts.len());

    for wire in &receipts {
        let r = match MicroReceipt::try_from(wire.clone()) {
            Ok(r) => r,
            Err(e) => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: format!("Wire conversion failed in builder: {:?}", e),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };

        total_spend = match total_spend.checked_add(r.metrics.spend) {
            Some(val) => val,
            None => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::RejectOverflow),
                    message: "Total spend overflow".to_string(),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };
        total_defect = match total_defect.checked_add(r.metrics.defect) {
            Some(val) => val,
            None => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::RejectOverflow),
                    message: "Total defect overflow".to_string(),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };

        // Authority must be preserved in slab compression (Patch 3)
        total_authority = match total_authority.checked_add(r.metrics.authority) {
            Some(val) => val,
            None => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::RejectOverflow),
                    message: "Total authority overflow".to_string(),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };

        let delta = match crate::semantic::PolicyEnvelopeRegistry::delta_hat(&r.step_type) {
            Ok((d, _)) => d,
            Err(e) => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: format!("Semantic lookup failed during slab build: {:?}", e),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };

        total_delta = match total_delta.checked_add(delta) {
            Some(val) => val,
            None => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::RejectOverflow),
                    message: "Total delta overflow".to_string(),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };
        leaves.push(r.chain_digest_next);
    }

    // Macro-defect check: total_defect must be >= sum(delta)
    if total_defect < total_delta {
        return BuildSlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectPolicyViolation),
            message: format!(
                "Macro defect bound violation: total_defect ({}) < total_delta ({})",
                total_defect, total_delta
            ),
            range_start: Some(first_wire.step_index),
            range_end: Some(last_wire.step_index),
            micro_count: Some(receipts.len() as u64),
            merkle_root: None,
            output: None,
            slab: None,
        };
    }

    let merkle_root = build_merkle_root(&leaves);

    let summary = SlabSummaryWire {
        total_spend: total_spend.to_string(),
        total_defect: total_defect.to_string(),
        authority: total_authority.to_string(),
        v_pre_first: first_wire.metrics.v_pre.clone(),
        v_post_last: last_wire.metrics.v_post.clone(),
    };

    let slab = SlabReceiptWire {
        schema_id: CanonRegistry::SLAB_V1_ID.to_string(),
        version: CanonRegistry::SLAB_V1_VERSION.to_string(),
        object_id: first_wire.object_id.clone(),
        canon_profile_hash: first_wire.canon_profile_hash.clone(),
        policy_hash: first_wire.policy_hash.clone(),
        range_start: first_wire.step_index,
        range_end: last_wire.step_index,
        micro_count: receipts.len() as u64,
        chain_digest_prev: first_wire.chain_digest_prev.clone(),
        chain_digest_next: last_wire.chain_digest_next.clone(),
        state_hash_first: first_wire.state_hash_prev.clone(),
        state_hash_last: last_wire.state_hash_next.clone(),
        merkle_root: merkle_root.to_hex(),
        summary,
    };

    BuildSlabResult {
        decision: Decision::SlabBuilt,
        code: None,
        message: "Slab built successfully".to_string(),
        range_start: Some(slab.range_start),
        range_end: Some(slab.range_end),
        micro_count: Some(slab.micro_count),
        merkle_root: Some(slab.merkle_root.clone()),
        output: None, // Filled by CLI
        slab: Some(slab),
    }
}
