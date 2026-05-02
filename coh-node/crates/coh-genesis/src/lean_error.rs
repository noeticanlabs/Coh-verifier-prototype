use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeanErrorKind {
    UnknownIdentifier,
    UnknownConstant,
    MissingImport,
    MissingLemma,
    TheoremNotInMathlib,
    LocalDerivationRequired,
    TheoremTooWeak,
    TypeMismatch,
    UnificationFailure,
    UnsolvedGoals,
    TacticFailed,
    RewriteFailed,
    SimpMadeNoProgress,
    LinarithFailed,
    RingFailed,
    OmegaFailed,
    InstanceSynthesisFailed,
    TerminationCheckFailed,
    PositivityFailed,
    FieldSimpFailed,
    BoundVariableIssue,
    NamespaceIssue,
    SyntaxError,
    Timeout,
    KernelError,
    UsesForbiddenShortcut,
    TheoremStatementLikelyTooWeak,
    Unknown,
}

pub fn classify_lean_error(raw: &str) -> LeanErrorKind {
    let s = raw.to_lowercase();

    if s.contains("unknown constant") || s.contains("unknown identifier") {
        LeanErrorKind::UnknownIdentifier
    } else if s.contains("failed to synthesize") {
        LeanErrorKind::InstanceSynthesisFailed
    } else if s.contains("type mismatch") {
        LeanErrorKind::TypeMismatch
    } else if s.contains("unsolved goals") {
        LeanErrorKind::UnsolvedGoals
    } else if s.contains("rewrite tactic failed") {
        LeanErrorKind::RewriteFailed
    } else if s.contains("simp made no progress") || s.contains("simp failed") {
        LeanErrorKind::SimpMadeNoProgress
    } else if s.contains("timeout") || s.contains("heartbeat") {
        LeanErrorKind::Timeout
    } else if s.contains("sorry") || s.contains("admit") || s.contains("axiom") {
        LeanErrorKind::UsesForbiddenShortcut
    } else if s.contains("expected command") || s.contains("unexpected token") {
        LeanErrorKind::SyntaxError
    } else {
        LeanErrorKind::Unknown
    }
}

pub struct LeanFailure {
    pub target: String,
    pub candidate_id: u64,
    pub tactic: String,
    pub raw_error: String,
    pub error_kind: LeanErrorKind,
    pub remaining_goals: usize,
    pub goal_fingerprint: String,
    pub imports_fingerprint: String,
    pub elapsed_ms: u64,
}
