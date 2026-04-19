import Coh.Prelude

namespace Coh.Contract

inductive RejectCode where
  | rejectSchema
  | rejectCanonProfile
  | rejectChainDigest
  | rejectStateHashLink
  | rejectNumericParse
  | rejectOverflow
  | rejectPolicyViolation
  | rejectSlabSummary
  | rejectSlabMerkle
  | rejectIntervalInvalid
  | rejectMissingSignature
  | rejectMissingObjectId
  | noProgressLoop
  | stateCycleDetected
  | retryBudgetExceeded
  | temporalDriftDetected
  | trajectoryCostExceeded
  | vacuousZeroReceipt
  | spendExceedsBalance
  | semanticTypeViolation
  | cumulativeDriftDetected
  | stepBudgetExceeded
  | timeBudgetExceeded
  | memoryBudgetExceeded
  | depthLimitExceeded
  -- GCCP compute-specific failures (Section 18)
  | rejectTempCap
  | rejectPowerCap
  | rejectQueueCap
  | rejectMemoryCap
  | rejectDefectCap
  | rejectBudget
  | rejectPredictorStale
  | rejectTelemetryStale
  | rejectRouteUnavailable
  | rejectPolicyClassMismatch
  deriving Repr, DecidableEq

end Coh.Contract
