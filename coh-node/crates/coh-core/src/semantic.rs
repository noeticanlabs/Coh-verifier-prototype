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

    /// Check if the receipt's defect is within the policy envelope: defect <= delta_hat
    ///
    /// DEFINITION: delta_hat is the MAXIMUM allowed defect envelope.
    /// The check enforces: defect <= delta_hat (observed <= maximum allowed)
    ///
    /// NOTE: This is the opposite direction from some legacy checks.
    /// We split the symbol as follows:
    /// - delta_hat (or Δ_max) = maximum admissible defect envelope
    /// - For coverage requirements, use a separate 'reserve' field
    pub fn verify_defect_bound(receipt: &MicroReceipt) -> Result<(), RejectCode> {
        let (delta_hat, _source) = Self::delta_hat(&receipt.step_type)?;
        if receipt.metrics.defect <= delta_hat {
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

    /// Oplax Subadditive Composition Law
    ///
    /// For composition R = R₂ ⊙ R₁:
    /// δ(R) ≤ δ(R₁) + δ(R₂)  (subadditivity)
    ///
    /// This implements the "Coh Category" composition law:
    /// The envelope defect of a composite trace cannot exceed
    /// the sum of component envelope defects.
    /// This accounts for hidden-fiber risk when traces are composed.
    ///
    /// # Arguments
    /// * `delta_hat_1` - Envelope defect of first trace R₁
    /// * `delta_hat_2` - Envelope defect of second trace R₂
    ///
    /// # Returns
    /// Upper bound on composite trace defect: δ(R₂ ⊙ R₁) ≤ δ(R₁) + δ(R₂)
    pub fn compose_delta_hat(delta_hat_1: u128, delta_hat_2: u128) -> u128 {
        // Simple additive upper bound
        // Lower bound would be max(delta_hat_1, delta_hat_2) for some compositions
        delta_hat_1.saturating_add(delta_hat_2)
    }

    /// Verify composite trace envelope is admissible
    ///
    /// Checks: declared_delta ≥ composed_delta
    /// where composed_delta = δ(R₂ ⊙ R₁)
    pub fn verify_composed_envelope(
        declared_delta_hat: u128,
        delta_hat_1: u128,
        delta_hat_2: u128,
    ) -> Result<(), RejectCode> {
        let composed = Self::compose_delta_hat(delta_hat_1, delta_hat_2);
        if declared_delta_hat >= composed {
            Ok(())
        } else {
            Err(RejectCode::SemanticEnvelopeViolation)
        }
    }
}
