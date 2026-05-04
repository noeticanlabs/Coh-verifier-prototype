pub mod code_patch;
pub mod ctrl_cohbit_adapter;
pub mod lean_proof;
pub mod mathlib_advisor;

pub use code_patch::CodePatchCandidate;
pub use ctrl_cohbit_adapter::{
    attempt_to_cohbit, build_receipt, compute_receipt_digest, tamper_and_redigest,
    verify_receipt_integrity, CtrlAccountingBudget, CtrlCohBitCandidate, CtrlCohTrajectory,
    CtrlObjectiveResult, CtrlRepairReceipt,
};
pub use lean_proof::{
    BenchmarkAudit, LeanErrorKind, PatchTransaction, PatchTransactionStatus,
    ProofCandidate as LeanProofCandidate, TransactionEntry, TransactionLog,
};
pub use mathlib_advisor::{MathlibAdvisorReport, MathlibStrategy};
