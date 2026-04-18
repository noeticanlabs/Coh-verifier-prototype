use crate::types::MicroReceiptWire;

/// Failure modes used to inject realistic invalid receipts that should
/// deterministically map to specific RejectCode outcomes under verify_micro().
#[derive(Clone, Copy, Debug)]
pub enum FailureMode {
    // Financial
    OverBudget,      // spend > v_pre -> RejectCode::SpendExceedsBalance
    MissingApproval, // arithmetic imbalance without authority -> RejectPolicyViolation

    // Agent
    TokenHallucination, // spend > v_pre or arithmetic imbalance
    StateCorruption,    // post-seal mutation -> RejectChainDigest
    HiddenToolFailure,  // arithmetic imbalance to reflect hidden failure

    // Ops
    Overtime,          // spend > v_pre
    MissingInspection, // drop signatures -> RejectMissingSignature
    InventoryCorruption, // arithmetic imbalance (v_post increases while spending)

                       // Reserved for chain-level extensions (not used in micro run)
                       // DoubleSpend,
                       // StepSkipped,
}

/// Injects a concrete failure into a MicroReceiptWire according to FailureMode
pub trait FailureInjector {
    fn inject(&self, receipt: &mut MicroReceiptWire, mode: FailureMode);
}
