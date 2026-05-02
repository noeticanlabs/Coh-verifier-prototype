use crate::lean_error::LeanErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairAction {
    TryNextCandidate,
    AddImport(String),
    DecomposeGoals,
    TryGoalSpecificTactics,
    TypeAlign,
    RewriteAlternate,
    SplitLemma,
    ReduceSearch,
    AddMissingHypothesis(crate::invariant_hunter::InvariantKind),
    InvokeInvariantLemma(String),
    StrengthenTheoremStatement,
    RejectPolicyViolation,
    Escalate,
}

pub fn choose_repair_action(kind: LeanErrorKind) -> RepairAction {
    match kind {
        LeanErrorKind::UnknownIdentifier | LeanErrorKind::UnknownConstant | LeanErrorKind::MissingImport => {
            RepairAction::AddImport("Mathlib.Tactic".into())
        }
        LeanErrorKind::UnsolvedGoals => RepairAction::DecomposeGoals,
        LeanErrorKind::TypeMismatch | LeanErrorKind::UnificationFailure => RepairAction::TypeAlign,
        LeanErrorKind::RewriteFailed => RepairAction::RewriteAlternate,
        LeanErrorKind::SimpMadeNoProgress => RepairAction::TryGoalSpecificTactics,
        LeanErrorKind::Timeout => RepairAction::ReduceSearch,
        LeanErrorKind::UsesForbiddenShortcut => RepairAction::RejectPolicyViolation,
        _ => RepairAction::TryNextCandidate,
    }
}

pub struct RepairBudget {
    pub max_candidates: usize,
    pub max_repair_depth: usize,
    pub max_time_ms: u64,
    pub max_import_additions: usize,
    pub max_split_lemmas: usize,
}

impl Default for RepairBudget {
    fn default() -> Self {
        Self {
            max_candidates: 64,
            max_repair_depth: 4,
            max_time_ms: 30_000,
            max_import_additions: 3,
            max_split_lemmas: 5,
        }
    }
}
