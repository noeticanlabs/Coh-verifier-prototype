//! APE - Adversarial / Exploratory Proposal Engine
//!
//! A deterministic, strategy-driven system that generates structured candidate states
//! for stress-testing Coh Wedge verification.
//!
//! ## Architecture
//!
//! ```mermaid
//! flowchart LR
//!     A[APE / LLM / External Source] --> B[Coh Wedge Verifier]
//!     B --> C[Decision: Accept or Reject]
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use ape::{engine::generate, proposal::Input, Strategy};
//! use coh_core::finalize_micro_receipt;
//! use coh_core::types::{MetricsWire, MicroReceiptWire, SignatureWire};
//!
//! let input = Input::from_micro(
//!     finalize_micro_receipt(MicroReceiptWire {
//!         schema_id: "coh.receipt.micro.v1".to_string(),
//!         version: "1.0.0".to_string(),
//!         object_id: "example.micro".to_string(),
//!         canon_profile_hash:
//!             "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09".to_string(),
//!         policy_hash: "0".repeat(64),
//!         step_index: 0,
//!         step_type: Some("example".to_string()),
//!         signatures: Some(vec![SignatureWire {
//!             signature: "sig-0000000000000000".to_string(),
//!             signer: "fixture-signer-0".to_string(),
//!             timestamp: 1_700_000_000,
//!             authority_id: Some("fixture-signer-0".to_string()),
//!             scope: Some("*".to_string()),
//!             expires_at: None,
//!         }]),
//!         state_hash_prev: "0".repeat(64),
//!         state_hash_next: "1".repeat(64),
//!         chain_digest_prev: "0".repeat(64),
//!         chain_digest_next: "0".repeat(64),
//!         metrics: MetricsWire {
//!             v_pre: "100".to_string(),
//!             v_post: "90".to_string(),
//!             spend: "10".to_string(),
//!             defect: "0".to_string(),
//!         },
//!     })
//!     .expect("example fixture should finalize"),
//! );
//!
//! let proposal = generate(Strategy::Mutation, &input, 42);
//! assert_eq!(proposal.strategy, Strategy::Mutation);
//! ```

pub mod adapter;
pub mod engine;
pub mod fixtures;
pub mod http;
pub mod pipeline;
pub mod proposal;
pub mod realdata;
pub mod seed;
pub mod strategies;

pub use adapter::{LlmAdapter, LlmResponse, MockLlmAdapter};
pub use engine::generate;
pub use fixtures::{load_chain, load_micro, load_slab, FixtureError};
pub use http::{execute_verified, ExecuteVerifiedRequest, SidecarResponse};
pub use pipeline::{run_pipeline, PipelineResult};
pub use proposal::Strategy;
pub use proposal::{Candidate, Proposal};
pub use realdata::{
    ensure_output_dir, generate_runtime_ai_chain, generate_runtime_ai_micro, load_ai_demo_chain,
    load_ai_demo_micro, load_dashboard_valid_chain, write_output_json,
};
pub use seed::SeededRng;
