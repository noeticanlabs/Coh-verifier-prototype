//! Generate AI workflow demo fixtures with proper digests

use coh_core::types::{MetricsWire, MicroReceiptWire};
use coh_core::{canon::*, hash::compute_chain_digest};
use std::convert::TryFrom;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn main() {
    // AI workflow states
    let state_hashes = [
        "a1b2c3d4e5f6789012345678901234567890123456789012345678901234", // state_0
        "b2c3d4e5f6a7890123456789012345678901234567890123456789012345", // state_1
        "c3d4e5f6a7b8901234567890123456789012345678901234567890123456", // state_2
        "d4e5f6a7b8c9012345678901234567890123456789012345678901234567", // state_3
        "e5f6a7b8c9d0123456789012345678901234567890123456789012345678", // state_4
    ];

    // Create step 0: TASK_RECEIVED -> PLAN_CREATED
    let mut receipt0 = create_receipt(
        0,
        &state_hashes[0],
        &state_hashes[1],
        "100",
        "88",
        "12",
        "0",
    );
    let digest0 = compute_digest(&receipt0);
    receipt0.chain_digest_next = digest0.clone();
    println!("Step 0 digest: {}", digest0);

    // Create step 1: PLAN_CREATED -> TOOL_CALLED
    let mut receipt1 = create_receipt(1, &state_hashes[1], &state_hashes[2], "88", "80", "7", "1");
    receipt1.chain_digest_prev = digest0.clone();
    let digest1 = compute_digest(&receipt1);
    receipt1.chain_digest_next = digest1.clone();
    println!("Step 1 digest: {}", digest1);

    // Create step 2: TOOL_CALLED -> TOOL_RESULT_APPLIED
    let mut receipt2 = create_receipt(2, &state_hashes[2], &state_hashes[3], "80", "68", "11", "0");
    receipt2.chain_digest_prev = digest1.clone();
    let digest2 = compute_digest(&receipt2);
    receipt2.chain_digest_next = digest2.clone();
    println!("Step 2 digest: {}", digest2);

    // Create step 3: TOOL_RESULT_APPLIED -> FINAL_RESPONSE_EMITTED
    let mut receipt3 = create_receipt(3, &state_hashes[3], &state_hashes[4], "68", "55", "12", "0");
    receipt3.chain_digest_prev = digest2.clone();
    let digest3 = compute_digest(&receipt3);
    receipt3.chain_digest_next = digest3.clone();
    println!("Step 3 digest: {}", digest3);

    println!("\n=== Valid micro receipt (step 0) ===");
    println!("{}", serde_json::to_string_pretty(&receipt0).unwrap());

    println!("\n=== Valid chain JSONL ===");
    let jsonl0 = serde_json::to_string(&receipt0).unwrap();
    let jsonl1 = serde_json::to_string(&receipt1).unwrap();
    let jsonl2 = serde_json::to_string(&receipt2).unwrap();
    let jsonl3 = serde_json::to_string(&receipt3).unwrap();
    println!("{}", jsonl0);
    println!("{}", jsonl1);
    println!("{}", jsonl2);
    println!("{}", jsonl3);

    println!("\n=== Invalid digest micro (step 0 tampered) ===");
    let mut tampered = receipt0.clone();
    tampered.chain_digest_next =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    println!("{}", serde_json::to_string_pretty(&tampered).unwrap());

    println!("\n=== Invalid state link (step 2 with wrong prev) ===");
    let mut bad_state = receipt2.clone();
    bad_state.state_hash_prev =
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string();
    println!("{}", serde_json::to_string_pretty(&bad_state).unwrap());
}

fn create_receipt(
    step: u64,
    prev_state: &str,
    next_state: &str,
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
) -> MicroReceiptWire {
    MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "agent.workflow.demo".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        state_hash_prev: prev_state.to_string(),
        state_hash_next: next_state.to_string(),
        chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
        },
    }
}

fn compute_digest(receipt: &MicroReceiptWire) -> String {
    let r = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(r.chain_digest_prev, &bytes).to_hex()
}
