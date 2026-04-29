pub mod engine;
pub mod rewrite;
pub mod store;
pub mod templates;
pub mod traits;

pub use engine::{NpeConfig, NpeEngine, NpeError, NpeProposal, NpeState, ProposalStatus};

#[cfg(feature = "npe-graph")]
pub use engine::{NpeEdge, NpeProposalGraph};

#[cfg(feature = "npe-store")]
pub use store::NpeStore;

#[cfg(feature = "npe-rewrite")]
pub use rewrite::NpeRewriter;

#[cfg(feature = "npe-parallel")]
pub use engine::parallel_score_proposals;

pub use traits::{NpeGenerator, NpeScorer, NpeVerifier};

use crate::phaseloom_lite::BoundaryReceiptSummary;

/// NPE Structural Memory Update
/// Returns the updated receipt with template information
pub fn enrich_receipt_with_template(
    mut receipt: BoundaryReceiptSummary,
    goal_text: &str,
) -> BoundaryReceiptSummary {
    if let Some(template) = templates::classify_coh_template(goal_text) {
        receipt.coh_template = Some(template);
    }
    receipt
}
