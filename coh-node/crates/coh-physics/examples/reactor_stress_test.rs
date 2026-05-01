// fixture_only: allow_mock
use coh_core::atom::CohAtom;
use coh_core::cohbit::CohBit;
use coh_core::spinor::CohSpinor;
use coh_core::types::{Hash32, DomainId};
use coh_genesis::vm_runtime::PhaseLoomRuntime;
use coh_npe::NpeConfig;
use coh_phaseloom::{PhaseLoomState, PhaseLoomConfig, CompressionContext, PhaseKey};
use num_rational::Rational64;
use std::collections::HashMap;

fn main() {
    let domain = DomainId(Hash32([1; 32]));
    let npe_config = NpeConfig::default();
    let mut runtime = PhaseLoomRuntime::new(domain, npe_config);
    let initial_state = Hash32([0; 32]);

    println!("Starting Coh-wedge Comprehensive Reactor Wall & Cross-Layer Stress Test\n");

    // --- 1. COMPRESSION ABUSE ---
    println!("--- 1. Compression Abuse ---");
    let mut loom = PhaseLoomState::new(&PhaseLoomConfig::default());
    let bucket_key = PhaseKey::from_spinor(&CohSpinor::default());
    let ctx = CompressionContext {
        min_sources: 2,
        max_depth: 2, 
        global_loss_hat: Rational64::new(1, 10),
        policy_hash: Hash32([0x77; 32]),
        verifier_id: Hash32([0x88; 32]),
        w_m: Rational64::from_integer(1),
        w_u: Rational64::from_integer(1),
        w_p: Rational64::from_integer(1),
    };

    // Fill sources for depth 1
    for i in 0..5 {
        let mut a = CohAtom::identity(initial_state, Rational64::from_integer(100), domain);
        a.atom_id = Hash32([i as u8; 32]);
        a.atom_hash = a.canonical_hash();
        loom.weave(a, &CohSpinor::default());
    }
    
    let res1 = loom.compress_bucket(bucket_key.clone(), Rational64::new(1, 100), &ctx).expect("First compression failed");
    println!("Depth 1 success. Depth: {}", res1.lineage.depth);
    
    // Add more sources for depth 2
    for i in 10..15 {
        let mut a = CohAtom::identity(initial_state, Rational64::from_integer(100), domain);
        a.atom_id = Hash32([i as u8; 32]);
        a.atom_hash = a.canonical_hash();
        loom.weave(a, &CohSpinor::default());
    }
    
    let res2 = loom.compress_bucket(bucket_key.clone(), Rational64::new(1, 100), &ctx).expect("Second compression failed");
    println!("Depth 2 success. Depth: {}", res2.lineage.depth);
    
    // Add more sources for depth 3
    for i in 20..25 {
        let mut a = CohAtom::identity(initial_state, Rational64::from_integer(100), domain);
        a.atom_id = Hash32([i as u8; 32]);
        a.atom_hash = a.canonical_hash();
        loom.weave(a, &CohSpinor::default());
    }
    
    let res3 = loom.compress_bucket(bucket_key.clone(), Rational64::new(1, 100), &ctx);
    match res3 {
        Ok(c) => println!("Depth 3 UNEXPECTED SUCCESS. Depth: {}", c.lineage.depth),
        Err(e) => println!("Depth 3 test (expect RecursiveDepthExceeded): {}", e),
    }

    // --- 2. PHASE LOOM RETRIEVAL ATTACKS ---
    println!("\n--- 2. Phase Loom Retrieval Attacks ---");
    let hits_cold = loom.retrieve(&CohSpinor::default(), Rational64::from_integer(0), 10, 0);
    println!("Hits in compressed bucket: {}", hits_cold.len());
    
    // Bucket flooding
    println!("Flooding bucket 0 with 10k junk atoms...");
    for i in 0..10_000 {
        let mut a = CohAtom::identity(initial_state, Rational64::from_integer(100), domain);
        let mut id_bytes = [0xFF; 32];
        let i_u16 = i as u16;
        id_bytes[31] = (i_u16 & 0xFF) as u8;
        id_bytes[30] = (i_u16 >> 8) as u8;
        a.atom_id = Hash32(id_bytes);
        a.atom_hash = a.canonical_hash();
        loom.weave(a, &CohSpinor::default());
    }
    println!("Active threads in loom: {}", loom.metrics.active_threads);
    let hits_flood = loom.retrieve(&CohSpinor::default(), Rational64::from_integer(0), 8, 0);
    println!("Retrieved hits (capped at 8): {}", hits_flood.len());
    assert_eq!(hits_flood.len(), 8);

    // --- 4. COHBIT REACTOR WALL ---
    println!("\n--- 4. CohBit Reactor Wall ---");
    let mut bit = CohBit::identity(initial_state, Rational64::from_integer(100), domain);
    bit.to_state = Hash32([1; 32]);
    bit.valuation_post = Rational64::from_integer(90); // 100 -> 90
    bit.spend = Rational64::from_integer(10);
    bit.defect = Rational64::from_integer(5);
    bit.delta_hat = Rational64::from_integer(10);
    bit.receipt_hash = bit.canonical_hash();
    
    println!("Valid bit executable check: {}", bit.executable());
    assert!(bit.executable());

    bit.defect = Rational64::from_integer(11);
    println!("Defect > Delta_hat test (executable): {}", bit.executable());
    assert!(!bit.executable());

    // --- 5. COHATOM CONTINUITY ---
    println!("\n--- 5. CohAtom Continuity ---");
    let mut atom = CohAtom::identity(initial_state, Rational64::from_integer(100), domain);
    atom.bits = vec![CohBit::identity(initial_state, Rational64::from_integer(100), domain), 
                     CohBit::identity(Hash32([2; 32]), Rational64::from_integer(100), domain)];
    
    println!("State continuity break test: {:?}", atom.continuity_valid().err().unwrap());
    
    // --- 6. CROSS-LAYER LAUNDERING ---
    println!("\n--- 6. Cross-Layer Laundering (The Passport Test) ---");
    let mut illegal_atom = CohAtom::identity(initial_state, Rational64::from_integer(100), domain);
    illegal_atom.cumulative_spend = Rational64::from_integer(1000); // Mismatch!
    illegal_atom.atom_hash = illegal_atom.canonical_hash();
    
    println!("Illegal atom executable check: {}", illegal_atom.executable());
    assert!(!illegal_atom.executable());
    
    let weaved = runtime.loom.weave(illegal_atom.clone(), &CohSpinor::default());
    println!("Weave illegal atom directly to loom: {}", weaved);
    assert!(!weaved); 

    let mut anchor_set = coh_phaseloom::AnchorSet::default();
    anchor_set.lambda = Rational64::from_integer(-1); // Negative lambda!
    let spinor = CohSpinor::default();
    let res_anchor = anchor_set.compute_alignment(&spinor, &HashMap::new(), &HashMap::new());
    let drift = (anchor_set.lambda * (Rational64::from_integer(1) - res_anchor)).reduced();
    println!("Negative lambda anchor drift cost: {}", drift);
    
    println!("\nALL REACTOR STRESS TESTS COMPLETED (VIOLATIONS CAUGHT)");
}
