//! Recombination Strategy
//!
//! Merges multiple valid states incorrectly to break chain continuity.

use crate::proposal::Candidate;
use crate::proposal::Input;
use crate::seed::SeededRng;
use coh_core::types::MicroReceiptWire;

/// Run recombination strategy - generates micro receipts with broken chain links
pub fn run(input: &Input, rng: &mut SeededRng) -> Candidate {
    // Instead of generating chains (which the demo doesn't verify),
    // generate micro receipts that have WRONG predecessor digests
    // This simulates "I came from a chain I wasn't part of"
    if let Some(ref micro) = input.base_micro {
        // Mutate the chain_digest_prev to point to wrong predecessor
        let mut broken = micro.clone();
        broken.chain_digest_prev = format!("{:064x}", rng.next() as u64); // Wrong digest!
                                                                          // Don't recompute chain_digest_next - this is the attack
        Candidate::Micro(broken)
    } else {
        // Generate a micro with broken chain link
        let mut sample = create_sample(rng, 0);
        sample.chain_digest_prev = "cafebabecafebabe".repeat(4); // Wrong predecessor
        Candidate::Micro(sample)
    }
}

fn recombine_chain(chain: &[MicroReceiptWire], rng: &mut SeededRng) -> Candidate {
    let mut new_chain: Vec<MicroReceiptWire> = Vec::new();

    // Simple 3-step chain with broken link at position 1
    if chain.len() >= 1 {
        // Step 0: copy first
        new_chain.push(chain[0].clone());

        // Step 1: swap state_hash_next with random garbage
        let mut step1 = if chain.len() > 1 {
            chain[1].clone()
        } else {
            create_sample(rng, 1)
        };
        step1.state_hash_next = "deadbeef".repeat(8); // Broken link
        step1.chain_digest_next = "0".repeat(64); // Valid digest but wrong state
        new_chain.push(step1);

        // Step 2: copy from whichever, but link will be broken
        if chain.len() > 2 {
            new_chain.push(chain[2].clone());
        } else {
            let mut step2 = create_sample(rng, 2);
            step2.chain_digest_prev = new_chain[1].chain_digest_next.clone();
            new_chain.push(step2);
        }
    }

    Candidate::Chain(new_chain)
}

fn make_broken_chain(wire: &MicroReceiptWire, rng: &mut SeededRng) -> Candidate {
    let mut step0 = wire.clone();
    step0.step_index = 0;

    let mut step1 = create_sample(rng, 1);
    step1.chain_digest_prev = step0.chain_digest_next.clone();
    step1.state_hash_prev = step0.state_hash_next.clone();

    // Break the link
    step1.state_hash_next = "badbadbad".repeat(8);
    step1.chain_digest_next = compute_digest(&step1);

    let mut step2 = create_sample(rng, 2);
    step2.chain_digest_prev = step1.chain_digest_next.clone();
    step2.state_hash_prev = step1.state_hash_next.clone();
    step2.chain_digest_next = compute_digest(&step2);

    Candidate::Chain(vec![step0, step1, step2])
}

fn create_sample(rng: &mut SeededRng, step: u64) -> MicroReceiptWire {
    let v_pre = 100u128 + (rng.next() as u128 % 1000);
    let spend = rng.next() as u128 % 50;
    let v_post = v_pre.saturating_sub(spend);

    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("ape.recombine.{}", step),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("recombination".to_string()),
        signatures: None,
        state_hash_prev: format!("{:064x}", step),
        state_hash_next: format!("{:064x}", step + 1),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: coh_core::types::MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: "0".to_string(),
        },
    };

    wire.chain_digest_next = compute_digest(&wire);
    wire
}

fn generate_valid_micro(rng: &mut SeededRng) -> Candidate {
    let step = rng.next() as u64;
    let v_pre = 100u128 + (rng.next() as u128 % 1000);
    let spend = rng.next() as u128 % 50;
    let v_post = v_pre.saturating_sub(spend);

    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("ape.recombine.{}", step),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("recombination".to_string()),
        signatures: None,
        state_hash_prev: format!("{:064x}", step),
        state_hash_next: format!("{:064x}", step + 1),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: compute_digest_raw(step, v_pre, v_post, spend),
        metrics: coh_core::types::MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: "0".to_string(),
        },
    };

    Candidate::Micro(wire)
}

fn compute_digest(wire: &MicroReceiptWire) -> String {
    use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
    use coh_core::hash::compute_chain_digest;
    use std::convert::TryFrom;

    if let Ok(r) = coh_core::types::MicroReceipt::try_from(wire.clone()) {
        let prehash = to_prehash_view(&r);
        if let Ok(bytes) = to_canonical_json_bytes(&prehash) {
            return compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();
        }
    }
    "0".repeat(64)
}

fn compute_digest_raw(step: u64, v_pre: u128, v_post: u128, spend: u128) -> String {
    use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
    use coh_core::hash::compute_chain_digest;
    use coh_core::types::MicroReceipt;
    use serde::Serialize;
    use std::convert::TryFrom;

    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("ape.recombine.{}", step),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("recombination".to_string()),
        signatures: None,
        state_hash_prev: format!("{:064x}", step),
        state_hash_next: format!("{:064x}", step + 1),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: coh_core::types::MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: "0".to_string(),
        },
    };

    if let Ok(r) = MicroReceipt::try_from(wire) {
        let prehash = to_prehash_view(&r);
        if let Ok(bytes) = to_canonical_json_bytes(&prehash) {
            return compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();
        }
    }
    "0".repeat(64)
}
