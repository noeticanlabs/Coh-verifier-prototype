//! External validation module: domain models, adapters, and generators
//!
//! Provides domain-aware workflow adapters to simulate realistic receipts
//! and inject invalid cases mapped to concrete RejectCode outcomes.

pub mod adapters;
pub mod domain;
pub mod generator;
pub mod logs;

pub use adapters::{AgentAdapter, FinancialAdapter, OpsAdapter};
pub use domain::{FailureInjector, FailureMode};
pub use generator::{run_external_validation_micro, ExtValidationReport};
pub use logs::{ingest_api_jsonl, ingest_cicd_jsonl, ingest_pipeline_jsonl, run_logs_validation};
