//! # Performance Benchmark
//!
//! Measures verification throughput for micro-receipts and chains.

use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire};
use coh_core::{build_slab, canon::*, hash::compute_chain_digest, verify_chain, verify_micro};
use std::convert::TryFrom;
use std::time::Instant;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn create_valid_receipt(step_index: u64, prev_digest: &str, prev_state: &str) -> MicroReceiptWire {
    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "benchmark.obj".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index,
        state_hash_prev: prev_state.to_string(),
        state_hash_next: prev_state.to_string(),
        chain_digest_prev: prev_digest.to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "80".to_string(),
            spend: "20".to_string(),
            defect: "0".to_string(),
        },
    };
    // Seal the receipt with proper digest
    let r = MicroReceipt::try_from(wire.clone()).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    wire.chain_digest_next = compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();
    wire
}

fn main() {
    println!("=== COHERENT VALIDATOR PERFORMANCE BENCHMARK ===\n");

    // Benchmark 1: Micro receipt verification
    println!("[1] Micro-receipt verification (10,000 iterations)");
    let receipt = create_valid_receipt(0, &"0".repeat(64), &"0".repeat(64));

    let start = Instant::now();
    for _ in 0..10_000 {
        let _ = verify_micro(receipt.clone());
    }
    let duration = start.elapsed();

    let ns_per_op = duration.as_nanos() as f64 / 10_000.0;
    let ops_per_sec = 10_000_000_000.0 / duration.as_nanos() as f64;

    println!("  Total time: {:?}", duration);
    println!("  ns/receipt: {:.0}", ns_per_op);
    println!("  receipts/sec: {:.0}", ops_per_sec);
    println!();

    // Benchmark 2: Chain verification (various lengths)
    for chain_len in [10, 100, 1000] {
        println!("[2] Chain verification ({} receipts)", chain_len);

        let mut receipts = Vec::with_capacity(chain_len);
        let mut prev_digest = "0".repeat(64);
        let mut prev_state = "0".repeat(64);

        for i in 0..chain_len {
            let r = create_valid_receipt(i as u64, &prev_digest, &prev_state);
            prev_digest = r.chain_digest_next.clone();
            prev_state = r.state_hash_next.clone();
            receipts.push(r);
        }

        let start = Instant::now();
        let result = verify_chain(receipts);
        let duration = start.elapsed();

        let ns_per_op = duration.as_nanos() as f64 / chain_len as f64;
        let ops_per_sec = 1_000_000_000.0 / duration.as_nanos() as f64 * chain_len as f64;

        println!("  Decision: {:?}", result.decision);
        println!("  Total time: {:?}", duration);
        println!("  ns/receipt: {:.0}", ns_per_op);
        println!("  receipts/sec: {:.0}", ops_per_sec);
        println!();
    }

    // Benchmark 3: Slab building
    println!("[3] Slab building (100 receipts)");
    let mut receipts = Vec::with_capacity(100);
    let mut prev_digest = "0".repeat(64);
    let mut prev_state = "0".repeat(64);

    for i in 0..100 {
        let r = create_valid_receipt(i as u64, &prev_digest, &prev_state);
        prev_digest = r.chain_digest_next.clone();
        prev_state = r.state_hash_next.clone();
        receipts.push(r);
    }

    let start = Instant::now();
    let result = build_slab(receipts);
    let duration = start.elapsed();

    println!("  Decision: {:?}", result.decision);
    println!("  Time: {:?}", duration);
    println!();

    println!("=== BENCHMARK COMPLETE ===");
}
