use crate::types::{MicroReceiptWire, MetricsWire, Decision, RejectCode, SignatureWire};
use crate::verify_micro;
use crate::trajectory::types::{witness_vector, WitnessStatus, ConstraintWitness};
use crate::canon::{EXPECTED_CANON_PROFILE_HASH};

fn dummy_sig() -> Vec<SignatureWire> {
    vec![SignatureWire {
        signer: "test".to_string(),
        signature: "sig".to_string(),
        timestamp: 0,
    }]
}

#[test]
fn test_vector_c4_policy_violation() {
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "test.c4".to_string(),
        canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
        policy_hash: "0".repeat(64),
        step_index: 10,
        step_type: None,
        signatures: Some(dummy_sig()),
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "120".to_string(), // VIOLATION: lhs (170) > rhs (100)
            spend: "50".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectPolicyViolation));

    let witnesses = witness_vector(&res);
    let c6_status = witnesses.iter().find(|(c, _)| *c == ConstraintWitness::C6Policy).unwrap().1;
    assert_eq!(c6_status, WitnessStatus::Fail);
}

#[test]
fn test_vector_c5_digest_mismatch() {
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "test.c5".to_string(),
        canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
        policy_hash: "0".repeat(64),
        step_index: 0,
        step_type: None,
        signatures: Some(dummy_sig()),
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "f".repeat(64), // WRONG DIGEST
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "100".to_string(),
            spend: "0".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectChainDigest));
}

#[test]
fn test_vector_c1_schema_error() {
    let wire = MicroReceiptWire {
        schema_id: "invalid".to_string(), // BAD SCHEMA
        version: "1.0.0".to_string(),
        object_id: "test.c1".to_string(),
        canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
        policy_hash: "0".repeat(64),
        step_index: 0,
        step_type: None,
        signatures: Some(dummy_sig()),
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "100".to_string(),
            spend: "0".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    // Schema check happens before profiles/sigs, but good to have them
}
