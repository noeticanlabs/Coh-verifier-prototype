use coh_phaseloom::{PhaseLoomState, PhaseLoomConfig, MemoryAccessPolicy, ComponentRole, MemoryTier, MemoryOp, AccessDecision};
use coh_npe::receipt::BoundaryReceiptSummary;
use coh_core::types::{Hash32, Signature};
use coh_core::atom::{CohAtom, AtomKind};
use std::time::{Instant, Duration};

fn main() {
    println!("--- Coh-Bit Runtime Performance Benchmarks ---");
    
    let mut state = PhaseLoomState::new(&PhaseLoomConfig::default());
    let receipt = BoundaryReceiptSummary::default();

    // 1. Verifier Hot-Path Benchmark (Mocked Law check)
    let start = Instant::now();
    for _ in 0..10000 {
        let v_pre = 100;
        let v_post = 90;
        let spend = 5;
        let defect = 0;
        let _ = v_post + spend <= v_pre + defect;
    }
    let duration = start.elapsed() / 10000;
    println!("[1] Verifier Hot-Path (Law Check) p50: {:?}", duration);

    // 2. Memory Access Policy Overhead
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = MemoryAccessPolicy::check(ComponentRole::Verifier, MemoryTier::Micro, MemoryOp::Read);
    }
    let duration = start.elapsed() / 10000;
    println!("[2] Access Policy Check Latency: {:?}", duration);

    // 3. PhaseLoom View Construction (RV)
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = state.project_rv_view(&receipt);
    }
    let duration = start.elapsed() / 10000;
    println!("[3] RV View Construction p50: {:?}", duration);

    // 4. PhaseLoom View Construction (GCCP)
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = state.project_gccp_view();
    }
    let duration = start.elapsed() / 10000;
    println!("[4] GCCP View Construction p50: {:?}", duration);

    // 5. Signature Validation Latency (Simulated)
    let signature = Signature(vec![0xAA; 64]);
    let payload = b"test_receipt_data";
    let start = Instant::now();
    for _ in 0..1000 {
        // Simulated Ed25519 cost (~100us)
        let mut hasher = sha2::Sha256::default();
        use sha2::Digest;
        hasher.update(payload);
        hasher.update(&signature.0);
        let _ = hasher.finalize();
    }
    let duration = start.elapsed() / 1000;
    println!("[5] Signature Validation Latency (Simulated): {:?}", duration);

    // 6. Chain Integrity Benchmark (1k chain)
    let start = Instant::now();
    let mut current_hash = Hash32([0; 32]);
    for i in 0..1000 {
        let mut hasher = sha2::Sha256::default();
        use sha2::Digest;
        hasher.update(current_hash.0);
        hasher.update((i as u32).to_be_bytes());
        current_hash = Hash32(hasher.finalize().into());
    }
    let duration = start.elapsed();
    println!("[6] Hash-Chain (1k) Replay Latency: {:?}", duration);
    
    println!("\n--- Benchmark Complete ---");
}
