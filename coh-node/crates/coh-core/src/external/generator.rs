use crate::types::{Decision, MicroReceiptWire};
use crate::verify_micro::verify_micro;

use super::adapters::DomainAdapter;
use super::domain::{FailureInjector, FailureMode};

#[derive(Debug, Clone)]
pub struct ExtValidationReport {
    pub total_valid: usize,
    pub total_invalid: usize,
    pub accepted_valid: usize,
    pub rejected_valid: usize,
    pub accepted_invalid: usize,
    pub rejected_invalid: usize,
}

impl ExtValidationReport {
    pub fn false_accept_rate(&self) -> f64 {
        if self.total_invalid == 0 {
            0.0
        } else {
            self.accepted_invalid as f64 / self.total_invalid as f64
        }
    }
    pub fn false_reject_rate(&self) -> f64 {
        if self.total_valid == 0 {
            0.0
        } else {
            self.rejected_valid as f64 / self.total_valid as f64
        }
    }
}

/// Build N valid receipts and M invalid receipts (by injecting the listed failure modes
/// in a round-robin fashion) and run verify_micro() on each. Returns a confusion-style
/// report.
pub fn run_external_validation_micro<A: DomainAdapter + FailureInjector + Copy>(
    adapter: A,
    valid_count: usize,
    invalid_count: usize,
    modes: &[FailureMode],
) -> ExtValidationReport {
    assert!(!modes.is_empty(), "at least one failure mode required");

    // Build valid chain seeds
    let mut valids: Vec<MicroReceiptWire> = Vec::with_capacity(valid_count);
    let mut prev_digest = "0".repeat(64);
    let mut prev_state = "0".repeat(64);
    for i in 0..valid_count {
        let r = adapter.build_valid(i as u64, &prev_digest, &prev_state);
        prev_digest = r.chain_digest_next.clone();
        prev_state = r.state_hash_next.clone();
        valids.push(r);
    }

    // Derive invalids by cloning latest valid and injecting failures
    let mut invalids: Vec<MicroReceiptWire> = Vec::with_capacity(invalid_count);
    let mut base = valids
        .last()
        .cloned()
        .unwrap_or_else(|| adapter.build_valid(0, &"0".repeat(64), &"0".repeat(64)));
    for i in 0..invalid_count {
        let mut r = base.clone();
        let mode = modes[i % modes.len()];
        adapter.inject(&mut r, mode);
        invalids.push(r);
    }

    let mut accepted_valid = 0usize;
    let mut rejected_valid = 0usize;
    for r in &valids {
        let res = verify_micro(r.clone());
        match res.decision {
            Decision::Accept => accepted_valid += 1,
            Decision::Reject => rejected_valid += 1,
            _ => rejected_valid += 1,
        }
    }

    let mut accepted_invalid = 0usize;
    let mut rejected_invalid = 0usize;
    for r in &invalids {
        let res = verify_micro(r.clone());
        match res.decision {
            Decision::Accept => accepted_invalid += 1,
            Decision::Reject => rejected_invalid += 1,
            _ => accepted_invalid += 1,
        }
    }

    ExtValidationReport {
        total_valid: valid_count,
        total_invalid: invalid_count,
        accepted_valid,
        rejected_valid,
        accepted_invalid,
        rejected_invalid,
    }
}
