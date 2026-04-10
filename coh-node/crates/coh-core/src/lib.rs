//! # Coherent Validator Core
//!
//! A deterministic state transition validator with cryptographic tamper detection.
//!
//! ## Core Features
//!
//! - **Micro-receipt verification**: Validates individual state transition receipts
//! - **Chain verification**: Validates linked sequences of receipts with continuity enforcement
//! - **Slab operations**: Builds and verifies compressed macro receipts
//! - **Tamper detection**: Cryptographic digest verification catches unauthorized modifications
//!
//! ## Architecture
//!
//! The validator operates in three layers:
//!
//! 1. **Micro layer** (`verify_micro`): Single receipt validation
//!    - Schema validation
//!    - Policy arithmetic (v_post + spend <= v_pre + defect)
//!    - Cryptographic digest verification
//!
//! 2. **Chain layer** (`verify_chain`): Multi-receipt sequence validation
//!    - Step index continuity
//!    - State hash linkage
//!    - Chain digest linkage
//!
//! 3. **Slab layer** (`build_slab`, `verify_slab`): Macro receipt operations
//!    - Merkle root computation
//!    - Aggregate accounting verification
//!    - Macro inequality enforcement
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use coh_core::{verify_micro, verify_chain, build_slab, verify_slab};
//! use coh_core::types::{MicroReceiptWire, SlabReceiptWire, MetricsWire};
//!
//! // Create a sample receipt (normally loaded from JSON)
//! let receipt = MicroReceiptWire {
//!     schema_id: "coh.receipt.micro.v1".to_string(),
//!     version: "1.0.0".to_string(),
//!     object_id: "demo".to_string(),
//!     canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09".to_string(),
//!     policy_hash: "0".repeat(64),
//!     step_index: 0,
//!     state_hash_prev: "0".repeat(64),
//!     state_hash_next: "0".repeat(64),
//!     chain_digest_prev: "0".repeat(64),
//!     chain_digest_next: "0".repeat(64), // Would be computed in practice
//!     metrics: MetricsWire {
//!         v_pre: "100".to_string(),
//!         v_post: "80".to_string(),
//!         spend: "20".to_string(),
//!         defect: "0".to_string(),
//!     },
//! };
//!
//! // Verify a single receipt
//! let result = verify_micro(receipt);
//! // match result.decision { ... }
//!
//! // Verify a chain (Vec<MicroReceiptWire>)
//! // let result = verify_chain(receipts);
//!
//! // Build a slab from chain
//! // let result = build_slab(receipts);
//!
//! // Verify a slab
//! // let slab = SlabReceiptWire { ... };
//! // let result = verify_slab(slab);
//! ```
//!
//! ## Modules
//!
//! - [`types`] - Data structures for receipts and results
//! - [`verify_micro`] - Single receipt verification
//! - [`verify_chain`] - Chain continuity verification  
//! - [`build_slab`] - Slab construction
//! - [`verify_slab`] - Slab verification
//! - [`canon`] - Canonicalization and serialization
//! - [`hash`] - Cryptographic digest computation
//! - [`merkle`] - Merkle tree operations

pub mod build_slab;
pub mod canon;
pub mod hash;
pub mod math;
pub mod merkle;
pub mod reject;
pub mod types;
pub mod vectors;
pub mod verify_chain;
pub mod verify_micro;
pub mod verify_slab;

// Re-export main functions for convenient API
pub use build_slab::build_slab;
pub use verify_chain::verify_chain;
pub use verify_micro::verify_micro;
pub use verify_slab::{verify_slab, verify_slab_with_leaves};

// Re-export types for convenience
pub use types::{BuildSlabResult, VerifyChainResult, VerifyMicroResult, VerifySlabResult};
pub use types::{Decision, MicroReceiptWire, RejectCode, SlabReceiptWire};
