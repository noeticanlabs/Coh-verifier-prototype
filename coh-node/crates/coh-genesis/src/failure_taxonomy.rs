use serde::{Deserialize, Serialize};

/// Layer of the NPE pipeline where the failure occurred
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureLayer {
    /// Failure at Coh pre-gate (before any engine runs)
    CohPre,
    /// Failure in Rust code generation or execution
    RustCoding,
    /// Failure in Lean syntax parsing
    LeanSyntax,
    /// Failure in Lean type elaboration
    LeanElaboration,
    /// Failure in Lean logical proof state
    LeanProof,
    /// Failure in Mathlib advisor/search
    MathlibAdvisor,
    /// Failure at Coh post-gate (admission policy)
    CohPost,
}

/// Specific Rust coding failures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RustFailure {
    ParseError,
    MissingSemicolon,
    UnclosedDelimiter,
    UnknownIdentifier(String),
    UnresolvedImport(String),
    TypeMismatch,
    TraitBoundUnsatisfied(String),
    BorrowAfterMove,
    MutableBorrowConflict,
    LifetimeError,
    PatternNonExhaustive,
    UnusedImportWarning,
    DeadCodeWarning,
    TestFailure(String),
    ClippyFailure,
    SerializationBreak,
    PublicApiBreak,
}

/// Lean syntax-level failures (the code doesn't parse)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeanSyntaxFailure {
    ParseError,
    InvalidBinderSyntax,
    InvalidExistsSyntax,
    MissingBy,
    MissingComma,
    InvalidTacticBlock,
    UnexpectedToken(String),
    UnknownCommand(String),
}

/// Lean elaboration failures (the code parses but doesn't type-check)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeanElabFailure {
    UnknownIdentifier(String),
    UnknownField(String),
    TypeMismatch,
    ApplicationTypeMismatch,
    FailedToSynthesizeInstance(String),
    AmbiguousCoercion,
    CannotInferImplicit,
    InvalidProjection,
    UniverseMismatch,
}

/// Lean proof-state failures (logical gaps)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeanProofFailure {
    UnsolvedGoals,
    GoalShapeMismatch,
    TacticFailed(String),
    // Domain-specific hints for the loop to learn from
    NeedLowerBoundHalf,
    NeedGreatestLowerBoundHalf,
    NeedApproximationLemma,
    NeedOrderContradiction,
    NeedWitnessConstruction,
    NeedSetExtensionality,
    NeedRewrite,
}

/// Coh governance and admission policy failures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernanceFailure {
    SorryIntroduced,
    AdmitIntroduced,
    UnauthorizedAxiomIntroduced,
    TheoremStatementChanged,
    DefinitionWeakened,
    ForbiddenImport,
    TooManyImports,
    ProofCostTooHigh,
    ReceiptMissing,
    StatementHashMismatch,
    ProofHashMismatch,
}

/// Mathlib advisor failures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MathlibFailure {
    LemmaNotFound(String),
    LemmaNameMismatch,
    NamespaceMismatch,
    SuggestedLemmaWrongShape,
    SuggestedLemmaNeedsTypeclass,
    ImportTooHeavy,
    ImportPolicyRejected,
    ImportDoesNotExposeLemma,
}

/// Severity of the failure
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureSeverity {
    /// Fatal error, strategy should be penalized heavily
    HardReject,
    /// Minor syntax or elab error, repairable by small tweaks
    Repairable,
    /// Almost worked, keep this candidate as a near-miss for learning
    UsefulNearMiss,
    /// Unknown impact
    Unknown,
}

/// Suggested repair strategies
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepairStrategy {
    SyntaxRepair,
    FieldNameRepair,
    CoercionRepair,
    TypeAnnotationRepair,
    ImportMinimization,
    MathlibLemmaSearch,
    HelperLemmaCreation,
    ContradictionProof,
    ApproximationLemma,
    RustBorrowRepair,
    RustTypeRepair,
    RustImportRepair,
    TestBehaviorRepair,
}

/// Combined failure kind
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureKind {
    Rust(RustFailure),
    LeanSyntax(LeanSyntaxFailure),
    LeanElab(LeanElabFailure),
    LeanProof(LeanProofFailure),
    Mathlib(MathlibFailure),
    Governance(GovernanceFailure),
    Other(String),
}

/// Unified Failure Report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FailureReport {
    pub candidate_id: String,
    pub target: String,
    pub layer: FailureLayer,
    pub kind: FailureKind,
    pub raw_error: String,
    pub normalized_message: String,
    pub retryable: bool,
    pub severity: FailureSeverity,
    pub suggested_repairs: Vec<RepairStrategy>,
}

impl FailureSeverity {
    pub fn reward_signal(&self) -> f64 {
        match self {
            FailureSeverity::HardReject => -1.0,
            FailureSeverity::Repairable => -0.1,
            FailureSeverity::UsefulNearMiss => 0.3,
            FailureSeverity::Unknown => 0.0,
        }
    }
}
