//! Policy Envelope Layer - Conservative Cost Bounds for Verification
//!
//! Implements policy-based envelope checks for the Coh inequality.
//!
//! ## Mathematical Context
//!
//! The V2 envelope condition requires:
//! δ(R) = sup_{ξ∈𝒢_R} W(ξ)  (true hidden cost)
//! d_f ≥ δ(R_f)                       (envelope defect bound)
//!
//! This module provides ŵδ(R) - a *conservative policy envelope* that:
//! - δ(R) ≤ ŵδ(R) for all traces R (kernel obligation)
//! - Is derived from a static registry, not certified hidden-fiber computation
//!
//! ## KERNEL OBLIGATION THEOREM
//! -/-
//! For all traces R: δ(R) ≤ ŵδ(R)
//!
//! Where:
//! - δ(R) = true hidden cost (supremum over hidden realizations)
//! - ŵδ(R) = policy envelope (static table value)
//!
//! Until hidden-fiber semantics are complete, this is a POLICY-LEVEL guarantee,
//! not a full envelope certificate.

use crate::types::{MicroReceipt, RejectCode};

/// Source of a policy envelope bound
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyEnvelopeSource {
    StaticTable,
    RegistryLookup,
    TrajectoryEngine,
    ExternalCertificate,
}

/// Registry for policy envelope values (ŵδ - conservative cost bounds)
///
/// IMPORTANT: This is a POLICY-LEVEL envelope, NOT a certified hidden-fiber computation.
/// - delta(R) = true hidden cost (supremum over hidden realizations)
/// - ŵdelta(R) = policy envelope (static table value)
///
/// KERNEL OBLIGATION: δ(R) ≤ ŵdelta(R) for all traces R
///
/// Until hidden-fiber semantics are complete, this provides only:
/// - A conservative static table bound
/// - NOT a full envelope certificate
pub struct PolicyEnvelopeRegistry;

/// DEPRECATED: Use PolicyEnvelopeRegistry instead
#[deprecated(since = "0.2.0", note = "Use PolicyEnvelopeRegistry instead")]
pub type SemanticRegistry = PolicyEnvelopeRegistry;

impl PolicyEnvelopeRegistry {
    /// Get the policy envelope value (ŵδ) for a given step type.
    ///
    /// This is a CONSERVATIVE POLICY BOUND, not the true hidden cost.
    /// The kernel obligation is: delta(R) ≤ ŵdelta(R) for all traces R.
    pub fn delta_hat(
        step_type: &Option<String>,
    ) -> Result<(u128, PolicyEnvelopeSource), RejectCode> {
        match step_type.as_deref() {
            Some("coh.step.identity") => Ok((0, PolicyEnvelopeSource::StaticTable)),
            Some("coh.step.transfer") => Ok((5, PolicyEnvelopeSource::RegistryLookup)),
            Some("coh.step.mint") => Ok((0, PolicyEnvelopeSource::StaticTable)),
            Some("coh.step.burn") => Ok((0, PolicyEnvelopeSource::StaticTable)),
            Some(_) | None => Err(RejectCode::SemanticEnvelopeMissing),
        }
    }

    /// Check if the receipt's defect dominates the policy envelope: defect ≥ ŵδ
    ///
    /// KERNEL OBLIGATION: This check enforces d_f ≥ ŵδ(R_f) ⊸ d_f ≥ δ(R_f)
    /// Only valid if the theorem δ(R) ≤ ŵδ(R) is proven.
    pub fn verify_defect_bound(receipt: &MicroReceipt) -> Result<(), RejectCode> {
        let (delta_hat, _source) = Self::delta_hat(&receipt.step_type)?;
        if receipt.metrics.defect >= delta_hat {
            Ok(())
        } else {
            Err(RejectCode::SemanticEnvelopeViolation)
        }
    }

    /// Check if a step is an identity transition.
    /// Lean Axiom: Identity traces have zero cost.
    pub fn is_identity(step_type: &Option<String>) -> bool {
        matches!(step_type, Some(t) if t == "coh.step.identity")
    }
}
