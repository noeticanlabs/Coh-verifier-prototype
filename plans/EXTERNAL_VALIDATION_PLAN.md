# External Validation Architecture Plan

## Goal
Advance external validation from 5/10 to production-ready by creating real-world workflow adapters.

## Current State
- Coh validates synthetic/mocked receipts correctly (0% FA/FR)
- External validation score: 5/10 (synthetic only, no real-world patterns)
- Existing: [`enterprise_benchmark.rs`](coh-node/crates/coh-core/examples/enterprise_benchmark.rs) has workflow generators but they produce valid data only

## Approach: Option A - Simulate Real Maintenance Workflows

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    External Validation Layer                    │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Financial   │  │    Agent     │  │        Ops           │  │
│  │  Adapter     │  │   Adapter    │  │      Adapter         │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│         │                 │                      │              │
│         ▼                 ▼                      ▼              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Workflow Domain Models                      │    │
│  │  - Transaction, Budget, Invoice (Financial)             │    │
│  │  - ToolCall, Decision, StateUpdate (Agent)              │    │
│  │  - WorkOrder, Inspection, Repair (Ops)                  │    │
│  └─────────────────────────┬───────────────────────────────┘    │
│                            │                                     │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Receipt Generator                            │    │
│  │  - ValidReceiptBuilder                                   │    │
│  │  - InvalidReceiptBuilder (failure injection)            │    │
│  └─────────────────────────┬───────────────────────────────┘    │
│                            │                                     │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Coh Verifier                                 │    │
│  │  - verify_micro()                                        │    │
│  │  - verify_chain()                                        │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Steps

### Step 1: Create External Validation Module

**Location**: `coh-node/crates/coh-core/src/external/`

**Files**:
- `mod.rs` - Module exports
- `domain.rs` - Domain models (Transaction, WorkOrder, ToolCall)
- `adapters.rs` - Workflow adapters with real-world semantics
- `generator.rs` - Receipt generation with failure injection
- `benchmark.rs` - Integration with enterprise benchmark

### Step 2: Financial Workflow Adapter

**Domain Patterns**:
- Budget tracking (v_pre = available funds, spend = transaction amount)
- Invoice processing (v_post = remaining budget)
- Reconciliation (authority = approved by, defect = discrepancy)

**Realistic Failure Modes**:
1. **Over budget** - spend > v_pre (creates money)
2. **Double spend** - same state_hash_prev different v_post
3. **Missing approval** - authority = 0 for high-value transaction
4. **Invoice duplication** - same object_id, different step_index

**Example**:
```rust
// Financial: $10,000 budget, $3,000 invoice
let receipt = financial_adapter.create_invoice(
    budget: 10_000,
    amount: 3_000,
    approval_level: 2,  // Manager approval
).build_valid();  // Valid: v_post = 7_000

// Inject hallucination: claim $8,000 spent (impossible)
let invalid = receipt.clone().with_hallucination(
    claimed_spend: 8_000  // v_post would be 2_000, but accounting is wrong
);
```

### Step 3: Agent/AI Workflow Adapter

**Domain Patterns**:
- Token budget (v_pre = tokens remaining, spend = tokens used)
- Tool calls (authority = tool succeeded, defect = tool failed)
- State machine transitions (chain_digest tracks history)

**Realistic Failure Modes**:
1. **Token hallucination** - claim more tokens used than possible
2. **State corruption** - state_hash_next doesn't match actual state
3. **Tool failure hidden** - defect = 0 but tool actually failed
4. **Chain break** - skip step_index or wrong chain_digest_prev

**Example**:
```rust
// Agent: 1000 token budget, used 150 tokens for retrieval
let receipt = agent_adapter.retrieve_data(
    tokens_available: 1000,
    tokens_used: 150,
    tool_success: true,
).build_valid();

// Inject tool failure hidden as success
let invalid = receipt.clone().with_hidden_failure(
    actual_defect: 50  // Tool had 50 unit defect
);
```

### Step 4: Ops/Maintenance Workflow Adapter

**Domain Patterns**:
- Work order lifecycle (v_pre = labor hours remaining)
- Parts inventory (spend = parts used)
- Inspection checkpoints (authority = inspector approval)

**Realistic Failure Modes**:
1. **Overtime** - claim more hours than available
2. **Missing inspection** - authority = 0 for safety-critical step
3. **Parts inventory corruption** - negative inventory
4. **Step skipped** - chain_digest_prev mismatch

**Example**:
```rust
// Ops: 40 hour work week, 8 hours used on inspection
let receipt = ops_adapter.inspection(
    hours_available: 40,
    hours_used: 8,
    inspector_id: "cert-12345",
    safety_critical: true,
).build_valid();

// Inject: skip inspection (safety violation)
let invalid = receipt.with_missing_inspection();
```

### Step 5: Failure Injection Framework

```rust
pub enum FailureMode {
    // Financial
    OverBudget,
    DoubleSpend,
    MissingApproval,
    DuplicateInvoice,
    
    // Agent
    TokenHallucination,
    StateCorruption,
    HiddenToolFailure,
    ChainBreak,
    
    // Ops
    Overtime,
    MissingInspection,
    InventoryCorruption,
    StepSkipped,
}

pub trait FailureInjector {
    fn inject(&self, receipt: &mut MicroReceiptWire, mode: FailureMode);
}
```

### Step 6: Benchmark Integration

Add to [`enterprise_benchmark.rs`](coh-node/crates/coh-core/examples/enterprise_benchmark.rs):

```rust
// New section: External Validation
println!("\n=== EXTERNAL VALIDATION: Real-World Workflows ===\n");

// Financial: 1000 valid + 100 injected failures
let financial_results = run_external_validation(
    FinancialAdapter::new(),
    1000,
    vec![FailureMode::OverBudget, FailureMode::MissingApproval],
    100,
);

// Agent: 1000 valid + 100 injected failures  
let agent_results = run_external_validation(
    AgentAdapter::new(),
    1000,
    vec![FailureMode::TokenHallucination, FailureMode::HiddenToolFailure],
    100,
);

// Ops: 1000 valid + 100 injected failures
let ops_results = run_external_validation(
    OpsAdapter::new(),
    1000,
    vec![FailureMode::Overtime, FailureMode::MissingInspection],
    100,
);

// Report
println!("External Validation Results:");
println!("============================");
println!("Financial: {}% FA, {}% FR", financial_results.false_accepts, financial_results.false_rejects);
println!("Agent:     {}% FA, {}% FR", agent_results.false_accepts, agent_results.false_rejects);
println!("Ops:       {}% FA, {}% FR", ops_results.false_accepts, ops_results.false_rejects);
```

## Expected Outcomes

| Metric | Current | Target |
|--------|---------|--------|
| External Validation Score | 5/10 | 9/10 |
| Workflow Types | 0 | 3 |
| Failure Modes Tested | 0 | 12 |
| Realistic Receipts | Synthetic only | Domain-aware |

## Files to Create/Modify

1. **New**: `coh-node/crates/coh-core/src/external/mod.rs`
2. **New**: `coh-node/crates/coh-core/src/external/domain.rs`
3. **New**: `coh-node/crates/coh-core/src/external/adapters.rs`
4. **New**: `coh-node/crates/coh-core/src/external/generator.rs`
5. **Modify**: `coh-node/crates/coh-core/examples/enterprise_benchmark.rs` - add external validation section
6. **Modify**: `coh-node/crates/coh-core/Cargo.toml` - add external module