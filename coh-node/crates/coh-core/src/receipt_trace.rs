// Copyright 2024 Cohere Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Signed Receipt Verification Trace
//!
//! Diagnostic helper for debugging H-class (signed receipt/chain) failures.
//! Provides visibility into the verification pipeline:
//!   canonical bytes → digest → signature → chain digest → sequence → decision

use serde::{Deserialize, Serialize};

/// Trace output for debugging signed receipt verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptVerificationTrace {
    /// Stage 1: Canonical bytes (hex)
    pub canonical_hex: Option<String>,

    /// Stage 2: Core digest (from canonical bytes)
    pub core_digest: Option<String>,

    /// Stage 3: Full digest (for V3 signed receipts)
    pub full_digest: Option<String>,

    /// Stage 4a: Chain digest previous (from prior receipt or genesis)
    pub chain_digest_prev: Option<String>,

    /// Stage 4b: Chain digest expected next (computed)
    pub chain_digest_expected_next: Option<String>,

    /// Stage 4c: Chain digest claimed next (from receipt)
    pub chain_digest_claimed_next: Option<String>,

    /// Stage 5a: Sequence expected (computed from guard)
    pub sequence_expected: Option<String>,

    /// Stage 5b: Sequence claimed (from receipt)
    pub sequence_claimed: Option<String>,

    /// Stage 6: Signature message (what was signed)
    pub signature_message: Option<String>,

    /// Stage 7: Verification result
    pub verification_result: Option<String>,

    /// Final decision
    pub decision: Option<String>,
}

impl Default for ReceiptVerificationTrace {
    fn default() -> Self {
        Self {
            canonical_hex: None,
            core_digest: None,
            full_digest: None,
            chain_digest_prev: None,
            chain_digest_expected_next: None,
            chain_digest_claimed_next: None,
            sequence_expected: None,
            sequence_claimed: None,
            signature_message: None,
            verification_result: None,
            decision: None,
        }
    }
}

impl ReceiptVerificationTrace {
    /// Create a new empty trace
    pub fn new() -> Self {
        Self::default()
    }

    /// Record canonical bytes stage
    pub fn with_canonical(mut self, hex: impl Into<String>) -> Self {
        self.canonical_hex = Some(hex.into());
        self
    }

    /// Record core digest stage
    pub fn with_core_digest(mut self, digest: impl Into<String>) -> Self {
        self.core_digest = Some(digest.into());
        self
    }

    /// Record full digest stage
    pub fn with_full_digest(mut self, digest: impl Into<String>) -> Self {
        self.full_digest = Some(digest.into());
        self
    }

    /// Record chain linkage stage
    pub fn with_chain_linkage(
        mut self,
        prev: impl Into<String>,
        expected: impl Into<String>,
        claimed: impl Into<String>,
    ) -> Self {
        self.chain_digest_prev = Some(prev.into());
        self.chain_digest_expected_next = Some(expected.into());
        self.chain_digest_claimed_next = Some(claimed.into());
        self
    }

    /// Record sequence stage
    pub fn with_sequence(
        mut self,
        expected: impl Into<String>,
        claimed: impl Into<String>,
    ) -> Self {
        self.sequence_expected = Some(expected.into());
        self.sequence_claimed = Some(claimed.into());
        self
    }

    /// Record signature verification
    pub fn with_signature(mut self, message: impl Into<String>, valid: bool) -> Self {
        self.signature_message = Some(message.into());
        self.verification_result = Some(if valid {
            "valid".to_string()
        } else {
            "invalid".to_string()
        });
        self
    }

    /// Record final decision
    pub fn with_decision(mut self, decision: impl Into<String>) -> Self {
        self.decision = Some(decision.into());
        self
    }

    /// Print the trace for debugging
    pub fn print(&self) {
        println!("\n=== RECEIPT VERIFICATION TRACE ===");

        if let Some(ref hex) = self.canonical_hex {
            println!("[1] Canonical bytes: {}...", &hex[..hex.len().min(64)]);
        }

        if let Some(ref dig) = self.core_digest {
            println!("[2] Core digest:      {}", dig);
        }

        if let Some(ref dig) = self.full_digest {
            println!("[3] Full digest:     {}", dig);
        }

        if let (Some(prev), Some(exp), Some(claim)) = (
            &self.chain_digest_prev,
            &self.chain_digest_expected_next,
            &self.chain_digest_claimed_next,
        ) {
            println!("[4] Chain linkage:");
            println!("    prev:     {}", prev);
            println!("    expected: {}", exp);
            println!("    claimed:  {}", claim);
            if exp == claim {
                println!("    ✓ MATCH");
            } else {
                println!("    ✗ MISMATCH");
            }
        }

        if let (Some(exp), Some(claim)) = (&self.sequence_expected, &self.sequence_claimed) {
            println!("[5] Sequence accumulator:");
            println!("    expected: {}", exp);
            println!("    claimed:  {}", claim);
            if exp == claim {
                println!("    ✓ MATCH");
            } else {
                println!("    ✗ MISMATCH");
            }
        }

        if let Some(ref msg) = self.signature_message {
            println!(
                "[6] Signature: message={} result={}",
                msg,
                self.verification_result.as_deref().unwrap_or("?")
            );
        }

        if let Some(ref dec) = self.decision {
            println!("\n>>> DECISION: {}", dec);
        }

        println!("====================================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_complete_pipeline() {
        let trace = ReceiptVerificationTrace::new()
            .with_canonical("abcdef123456...")
            .with_core_digest("abc123")
            .with_full_digest("def456")
            .with_chain_linkage("prev789", "expectedABC", "claimedDEF")
            .with_sequence("seq001", "seq002")
            .with_signature("sigmsg", false)
            .with_decision("Reject");

        trace.print();

        assert!(trace.chain_digest_expected_next != trace.chain_digest_claimed_next);
        assert!(trace.sequence_expected != trace.sequence_claimed);
    }
}
