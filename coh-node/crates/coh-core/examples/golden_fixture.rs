//! # Golden Fixture Generator
//!
//! Generates valid receipts with correct digests for testing

use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire};
use coh_core::{canon::*, hash::compute_chain_digest};
use std::convert::TryFrom;

fn main() {
    let valid_profile = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

    // Create a valid receipt
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "golden_valid_001".to_string(),
        canon_profile_hash: valid_profile.to_string(),
        policy_hash: "0".repeat(64),
        step_index: 0,
        step_type: Some("test".to_string()),
        signatures: Some(vec![coh_core::types::SignatureWire {
            signature: "sig-golden-001".to_string(),
            signer: "golden-signer".to_string(),
            timestamp: 1700000000,
        }]),
        state_hash_prev: "0".repeat(64),
        state_hash_next: "1".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64), // Will compute
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "80".to_string(),
            spend: "15".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    // Compute the correct digest
    let r = MicroReceipt::try_from(wire.clone()).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    let computed_digest = compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();

    println!("=== GOLDEN VALID RECEIPT ===");
    println!("Computed chain_digest_next: {}", computed_digest);
    println!();
    println!("JSON:");
    println!("{{");
    println!("  \"schema_id\": \"{}\",", wire.schema_id);
    println!("  \"version\": \"{}\",", wire.version);
    println!("  \"object_id\": \"{}\",", wire.object_id);
    println!("  \"canon_profile_hash\": \"{}\",", wire.canon_profile_hash);
    println!("  \"policy_hash\": \"{}\",", wire.policy_hash);
    println!("  \"step_index\": {},", wire.step_index);
    println!("  \"signatures\": [");
    println!("    {{");
    println!(
        "      \"signature\": \"{}\",",
        wire.signatures.as_ref().unwrap()[0].signature
    );
    println!(
        "      \"signer\": \"{}\",",
        wire.signatures.as_ref().unwrap()[0].signer
    );
    println!(
        "      \"timestamp\": {}",
        wire.signatures.as_ref().unwrap()[0].timestamp
    );
    println!("    }}");
    println!("  ],");
    println!("  \"state_hash_prev\": \"{}\",", wire.state_hash_prev);
    println!("  \"state_hash_next\": \"{}\",", wire.state_hash_next);
    println!("  \"chain_digest_prev\": \"{}\",", wire.chain_digest_prev);
    println!("  \"chain_digest_next\": \"{}\",", computed_digest);
    println!("  \"metrics\": {{");
    println!("    \"v_pre\": \"{}\",", wire.metrics.v_pre);
    println!("    \"v_post\": \"{}\",", wire.metrics.v_post);
    println!("    \"spend\": \"{}\",", wire.metrics.spend);
    println!("    \"defect\": \"{}\",", wire.metrics.defect);
    println!("    \"authority\": \"{}\"", wire.metrics.authority);
    println!("  }}");
    println!("}}");
    println!();

    // Now create an invalid one - tampered digest
    println!("=== GOLDEN INVALID (Tampered Digest) ===");
    let mut invalid_wire = wire.clone();
    invalid_wire.object_id = "golden_invalid_001".to_string();
    invalid_wire.chain_digest_next = "deadbeef".repeat(8); // Invalid!
    println!("JSON:");
    println!("{{");
    println!("  \"schema_id\": \"{}\",", invalid_wire.schema_id);
    println!("  \"version\": \"{}\",", invalid_wire.version);
    println!("  \"object_id\": \"{}\",", invalid_wire.object_id);
    println!(
        "  \"canon_profile_hash\": \"{}\",",
        invalid_wire.canon_profile_hash
    );
    println!("  \"policy_hash\": \"{}\",", invalid_wire.policy_hash);
    println!("  \"step_index\": {},", invalid_wire.step_index);
    println!("  \"signatures\": [");
    println!("    {{");
    println!(
        "      \"signature\": \"{}\",",
        invalid_wire.signatures.as_ref().unwrap()[0].signature
    );
    println!(
        "      \"signer\": \"{}\",",
        invalid_wire.signatures.as_ref().unwrap()[0].signer
    );
    println!(
        "      \"timestamp\": {}",
        invalid_wire.signatures.as_ref().unwrap()[0].timestamp
    );
    println!("    }}");
    println!("  ],");
    println!(
        "  \"state_hash_prev\": \"{}\",",
        invalid_wire.state_hash_prev
    );
    println!(
        "  \"state_hash_next\": \"{}\",",
        invalid_wire.state_hash_next
    );
    println!(
        "  \"chain_digest_prev\": \"{}\",",
        invalid_wire.chain_digest_prev
    );
    println!(
        "  \"chain_digest_next\": \"{}\",",
        invalid_wire.chain_digest_next
    );
    println!("  \"metrics\": {{");
    println!("    \"v_pre\": \"{}\",", invalid_wire.metrics.v_pre);
    println!("    \"v_post\": \"{}\",", invalid_wire.metrics.v_post);
    println!("    \"spend\": \"{}\",", invalid_wire.metrics.spend);
    println!("    \"defect\": \"{}\",", invalid_wire.metrics.defect);
    println!("    \"authority\": \"{}\"", invalid_wire.metrics.authority);
    println!("  }}");
    println!("}}");
}
