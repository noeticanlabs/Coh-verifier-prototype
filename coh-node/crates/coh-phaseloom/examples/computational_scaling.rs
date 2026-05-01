use coh_phaseloom::{PhaseLoomState, PhaseLoomConfig};
use coh_core::atom::CohAtom;
use coh_core::types::{Hash32, DomainId};
use coh_core::spinor::CohSpinor;
use num_rational::Rational64;
use std::time::Instant;

fn main() {
    println!("--- PhaseLoom Computational Scaling Benchmark ---");
    
    let mut state = PhaseLoomState::new(&PhaseLoomConfig::default());
    let atom_count = 100_000;
    
    let domain = DomainId(Hash32([0x11; 32]));
    let verifier = Hash32([0x22; 32]);
    let policy = Hash32([0x33; 32]);
    let valuation = Rational64::from_integer(100);

    println!("Populating PhaseLoom with {} atoms...", atom_count);
    let start = Instant::now();
    for i in 0..atom_count {
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0..4].copy_from_slice(&(i as u32).to_be_bytes());
        let hash = Hash32(hash_bytes);
        
        let atom = CohAtom::certified_identity(
            hash,
            valuation,
            domain,
            verifier,
            policy
        );
        
        let mut spinor = CohSpinor::default();
        spinor.state_hash = atom.final_state;
        spinor.norm = Rational64::from_integer(1); // SA1 alignment
        
        if !state.weave(atom, &spinor) {
             panic!("Failed to weave atom at index {}", i);
        }
        
        // Occasionally register as anchor to grow the index
        if i % 1000 == 0 {
            let _ = state.anchor_set.register_atom_anchor(&state.thread_store.get(&hash).unwrap().atom);
        }
    }
    println!("Population completed in {:?}", start.elapsed());

    // 1. Indexed Retrieval Benchmark
    println!("\n[1] Indexed Alignment Check (compute_alignment)");
    let mut test_hash_bytes = [0u8; 32];
    test_hash_bytes[0..4].copy_from_slice(&(50000u32).to_be_bytes());
    let mut test_spinor = CohSpinor::default();
    test_spinor.state_hash = Hash32(test_hash_bytes);
    test_spinor.norm = Rational64::from_integer(1);
    
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = state.spinor_anchor_result(&test_spinor);
    }
    let avg_retrieval = start.elapsed() / 10000;
    println!("Average Alignment Check Latency (100k atoms): {:?}", avg_retrieval);
    
    // 2. Projected View Benchmark
    println!("\n[2] Governed View Projection (Πc)");
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = state.project_gccp_view();
    }
    println!("Average GCCP View Projection Latency: {:?}", start.elapsed() / 10000);
}
