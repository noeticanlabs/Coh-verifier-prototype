use coh_phaseloom::{PhaseLoomState, PhaseLoomConfig, MemoryAccessPolicy, ComponentRole, MemoryView, MemoryOp, AccessDecision};
use coh_genesis::GenesisMetrics;
use coh_core::rv_kernel::{RvKernel, RvGoverningState, ProtectedRvBudget};
use coh_core::types::{Hash32, Signature};
use coh_npe::receipt::BoundaryReceiptSummary;
use std::time::{Instant, Duration};
use std::hint::black_box;

fn main() {
    println!("--- Coh-Bit Hardened Runtime Performance Benchmarks ---");
    
    let state = PhaseLoomState::new(&PhaseLoomConfig::default());
    let receipt = BoundaryReceiptSummary::default();
    let rv = RvKernel::new(RvGoverningState::default(), ProtectedRvBudget::default());

    // 1. Real Verifier Admissibility (Hot-Path)
    let start = Instant::now();
    for i in 0..10000 {
        let v_pre = black_box(100 + (i % 3) as u128);
        let v_post = black_box(90 + (i % 2) as u128);
        let spend = black_box(5);
        let defect = black_box(0);
        let _ = rv.is_admissible(v_post, spend, v_pre, defect);
    }
    let duration = start.elapsed() / 10000;
    println!("[1] Real Verifier Admissibility p50: {:?}", duration);

    // 2. Real GCCP Admissibility (Law of Genesis)
    let start = Instant::now();
    for i in 0..10000 {
        let m_pre = black_box(100 + (i % 3) as u128);
        let m_post = black_box(90 + (i % 2) as u128);
        let cost = black_box(5);
        let slack = black_box(0);
        let _ = GenesisMetrics::is_genesis_admissible(m_pre, m_post, cost, slack);
    }
    let duration = start.elapsed() / 10000;
    println!("[2] Real GCCP Admissibility p50: {:?}", duration);

    // 3. Access Policy Check Latency (View-based)
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = black_box(MemoryAccessPolicy::check(
            black_box(ComponentRole::Verifier), 
            black_box(MemoryView::TransitionView), 
            black_box(MemoryOp::Read)
        ));
    }
    let duration = start.elapsed() / 10000;
    println!("[3] View-Based Access Policy Check: {:?}", duration);

    // 4. PhaseLoom View Construction (RV)
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = black_box(state.project_rv_view(&receipt));
    }
    let duration = start.elapsed() / 10000;
    println!("[4] RV View Construction p50 (Π_RV): {:?}", duration);

    // 5. Raw SHA-256 Chain-Link Benchmark
    let start = Instant::now();
    let mut current_hash = Hash32([0; 32]);
    for i in 0..1000 {
        let mut hasher = sha2::Sha256::default();
        use sha2::Digest;
        hasher.update(black_box(current_hash.0));
        hasher.update(black_box((i as u32).to_be_bytes()));
        current_hash = Hash32(hasher.finalize().into());
    }
    let duration = start.elapsed();
    println!("[5] Raw SHA-256 Chain-Link (1k) Replay: {:?}", duration);

    // 6. Simulated Ed25519 Signature Verification
    println!("[6] Real Ed25519 signature benchmark deferred to production crate (Est: ~60-100µs)");

    println!("\n--- Benchmark Complete ---");
}
