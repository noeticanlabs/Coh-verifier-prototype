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
//! use ape::{engine::generate, Strategy, seed::SeededRng};
//!
//! let rng = SeededRng::new(42);
//! let proposal = generate(Strategy::Mutation, &input, rng);
//! ```

pub mod adapter;
pub mod engine;
pub mod fixtures;
pub mod pipeline;
pub mod proposal;
pub mod seed;
pub mod strategies;

pub use adapter::{LlmAdapter, LlmResponse, MockLlmAdapter};
pub use engine::generate;
pub use fixtures::{load_chain, load_micro, load_slab, FixtureError};
pub use pipeline::{run_pipeline, PipelineResult};
pub use proposal::Strategy;
pub use proposal::{Candidate, Proposal};
pub use seed::SeededRng;
