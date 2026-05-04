# CTRL-to-CohBit Adapter Plan

## Current Status

CTRL operates as a proof repair engine. To make it a **CohBit-governed repair actor**, we need a bridge module.

## Goal

```text
CTRL repair attempt = proposed state transition
CohBit decides whether that transition is admissible
```

## The Full Loop

```text
TheoremFailure
  ↓
CTRL diagnosis
  ↓
RepairCandidate ranked
  ↓
CohBit candidate receipt generated
  ↓
Accounting law check
  ↓
PatchTransaction in temp file
  ↓
LeanWorker verifies
  ↓
V3 receipt canonical digest
  ↓
sequence accumulator update
  ↓
CohBit emitted
  ↓
audit trail / dashboard display
```

## Architecture

| Coh term | CTRL meaning |
| --- | --- |
| v_pre | theorem repair budget before attempt |
| v_post | remaining proof confidence budget |
| spend | tactic cost, runtime, complexity, file mutation |
| defect | allowed proof uncertainty / known gap |
| authority | explicit permission to spend more budget |

Admissibility: `v_post + spend ≤ v_pre + defect + authority`

## Implementation Plan

### 1. New Module: ctrl_cohbit_adapter.rs

**Location**: `coh-node/crates/coh-npe/src/tools/ctrl_cohbit_adapter.rs`

**Components**:

- `CtrlRepairReceipt` - bridges attempt to CohBit
- `CtrlAccountingBudget` - CTRL interpretation of accounting law
- `CtrlObjectiveResult` - Lean result mapped to Coh objective
- `CtrlCohBitCandidate` - full candidate with receipt + accounting
- `CtrlCohTrajectory` - audit trail mapped toCoh trajectory

### 2. Key Types

```rust
pub struct CtrlRepairReceipt {
    pub theorem_name: String,
    pub theorem_hash_pre: String,
    pub theorem_hash_post: String,
    pub candidate_hash: String,
    pub tactic_hash: String,
    pub lean_result_hash: String,
    pub audit_hash: String,
    pub spend: u128,
    pub defect_reserve: u128,
    pub authority: u128,
    pub sequence_accumulator: String,
}
```

```rust
pub struct CtrlAccountingBudget {
    pub pre_budget: u128,    // v_pre
    pub post_budget: u128,   // v_post
    pub spend: u128,
    pub defect: u128,       // defect reserve
    pub authority: u128,
}
```

### 3. Convert Functions

```
attempt_to_cohbit: CtrlObjectiveResult + CtrlRepairReceipt + Accounting → CtrlCohBitCandidate

Only successful attempts become CohBits.
Failed attempts remain audit metadata.
```

### 4. Integration Points

1. Import in `tools/mod.rs`
2. Use existing `current_timestamp()` from lean_proof.rs
3. Bridge to existing CohBit chain accumulation

## Risk Notes

- Need to ensure accounting law correctly interprets CTRL costs
- Sequence accumulator needs proper hashing (currently simplified)
- Dashboard display not yet integrated

## Next Module

```text
ctrl_cohbit_adapter.rs
  ↓
verify_micro_v3 compatible receipt
  ↓
append to chain
```

This closes the loop: CTRL repairs → CohBit certifies → trajectory tracks.