use super::*;
use crate::types::{Hash32, RvStatus, Signature};
use num_rational::Rational64;

fn build_test_atom() -> CohAtom {
    // 1. Build a valid executable trajectory
    let mut atom = CohAtom::default();
    atom.atom_id = Hash32([0x11; 32]);
    atom.domain = DomainId(Hash32([0x55; 32]));

    let mut bit1 = CohBit::default();
    bit1.from_state = Hash32([0xaa; 32]);
    bit1.to_state = Hash32([0xbb; 32]);
    bit1.bit_id = Hash32([0x01; 32]);
    bit1.action_hash = Hash32([0x01; 32]);
    bit1.rv_status = RvStatus::Accept;
    bit1.signature = Signature(vec![0x11; 64]);
    bit1.valuation_pre = Rational64::from_integer(100);
    bit1.valuation_post = Rational64::from_integer(90);
    bit1.spend = Rational64::from_integer(10);
    bit1.step_index = 0;
    bit1 = bit1.finalize_hashes();

    let mut bit2 = CohBit::default();
    bit2.from_state = Hash32([0xbb; 32]);
    bit2.to_state = Hash32([0xcc; 32]);
    bit2.bit_id = Hash32([0x02; 32]);
    bit2.action_hash = Hash32([0x02; 32]);
    bit2.rv_status = RvStatus::Accept;
    bit2.signature = Signature(vec![0x22; 64]);
    bit2.valuation_pre = Rational64::from_integer(90);
    bit2.valuation_post = Rational64::from_integer(80);
    bit2.spend = Rational64::from_integer(10);
    bit2.prev_receipt_hash = Some(bit1.receipt_hash);
    bit2.chain_digest_pre = bit1.chain_digest_post;
    bit2.step_index = 1;
    bit2 = bit2.finalize_hashes();

    atom.bits = vec![bit1, bit2];
    atom.initial_state = Hash32([0xaa; 32]);
    atom.final_state = Hash32([0xcc; 32]);
    atom.cumulative_spend = Rational64::from_integer(20);
    atom.cumulative_defect = Rational64::from_integer(0);
    
    // 2. Compute hashes
    atom.atom_hash = atom.canonical_hash();
    atom
}

#[test]
fn test_refinery_compression_executable_to_summary() {
    let mut atom = build_test_atom();
    assert_eq!(atom.kind, AtomKind::ExecutableTrajectory);
    assert!(!atom.bits.is_empty());
    assert!(atom.compression_certificate.is_none());

    // Run Refinery
    atom.compress().expect("Compression should succeed for valid trajectory");

    // Verify Summary State
    assert_eq!(atom.kind, AtomKind::SummaryTrajectory);
    assert!(atom.bits.is_empty());
    assert!(atom.compression_certificate.is_some());
    
    // O(1) verify summary
    assert!(atom.retrieval_valid());
}

#[test]
fn test_refinery_compression_preserves_valuation_boundaries() {
    let mut atom = build_test_atom();
    let initial_val = atom.initial_valuation();
    let final_val = atom.final_valuation();

    atom.compress().unwrap();

    assert_eq!(atom.initial_valuation(), initial_val);
    assert_eq!(atom.final_valuation(), final_val);
}

#[test]
fn test_refinery_rejects_non_executable_compression() {
    let mut atom = build_test_atom();
    atom.cumulative_spend = Rational64::from_integer(999); // Corrupt it
    
    let res = atom.compress();
    assert!(res.is_err());
}

#[test]
fn test_summary_trajectory_not_mutable() {
    let mut atom = build_test_atom();
    atom.compress().unwrap();
    
    assert!(!atom.mutation_valid());
}
