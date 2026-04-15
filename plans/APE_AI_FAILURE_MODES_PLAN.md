# APE AI Failure Mode Strategies Plan

## Overview
Extend APE to target AI failure modes - not just structural corruption, but behavior that breaks real intelligent systems.

## New Strategy Set

### 1. Specification Gaming
- **What**: Satisfies explicit rules, violates intent
- **Example**: Receipt passes invariants but encodes wrong outcome (reward maximized on wrong target)
- **Target**: Intent vs formal rules gap

### 2. Distribution Shift  
- **What**: Pushes to rare edge distributions, unseen combinations
- **Example**: Valid values in weird combinations, timing anomalies
- **Target**: Robustness under non-typical states

### 3. Temporal Drift
- **What**: Each step locally valid, but global behavior drifts into invalidity
- **Example**: Small consistent bias accumulates, gradual divergence
- **Target**: Long-horizon integrity, accumulation effects

### 4. Ambiguity Exploitation
- **What**: Exploits undefined fields, optional fields, multi-interpretation structures
- **Example**: Same receipt interpreted differently by verifier vs generator
- **Target**: Schema ambiguity, parsing inconsistencies

### 5. Adversarial Alignment
- **What**: Appears highly aligned, passes superficial checks, hides deeper violation
- **Example**: High-confidence valid-looking receipt with subtle inconsistency buried deep
- **Target**: Reliance on shallow signals, overconfidence in validation

## Comparison with Original 5

| Original | Attack Type | Layer |
|----------|------------|-------|
| Mutation | Local corruption | R (receipt) |
| Recombination | Structural corruption | H (history) |
| Violation | Direct rule breaking | C (constraints) |
| Overflow | Bounds breaking | X (bounds) |
| Contradiction | Internal inconsistency | R (receipt) |

| New | Attack Type | Layer |
|-----|------------|-------|
| Specification Gaming | Intent violation | Intent vs formal |
| Distribution Shift | Domain violation | X (state distributions) |
| Temporal Drift | Time-based failure | H (history accumulation) |
| Ambiguity Exploitation | Definition gaps | R/C schema |
| Adversarial Alignment | Deceptive validity | Shallow vs deep |

## Implementation Priority

### Phase 1: Specification Gaming + Temporal Drift (highest impact)
1. **Specification Gaming**: 
   - Generate receipts that satisfy math but encode wrong "meaning"
   - Example: v_post = 0 (valid per v_post + spend <= v_pre) but represents "did nothing useful"

2. **Temporal Drift**:
   - Generate chains where each receipt is valid, but chain-level property drifts
   - Example: Each receipt individually valid, but cumulative spend exceeds total v_pre

### Phase 2: Distribution Shift + Ambiguity (medium impact)
3. **Distribution Shift**:
   - Generate extreme numeric edge cases (min/max values, overflow boundaries)
   - Example: v_pre very small or very large

4. **Ambiguity Exploitation**:
   - Test optional fields being missing vs present
   - Test different field ordering

### Phase 3: Adversarial Alignment (requires good baseline generation)
5. **Adversarial Alignment**:
   - Generate "ideal looking" receipts with hidden defects
   - Requires non-trivially correct baseline first

## Implementation in Strategy Enum

Add to `ape/src/proposal.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    // Original 5
    Mutation,
    Recombination, 
    Violation,
    Overflow,
    Contradiction,
    // New AI failure modes (Phase 1-3)
    SpecificationGaming,
    DistributionShift,
    TemporalDrift,
    AmbiguityExploitation,
    AdversarialAlignment,
}
```

## Demo Output Target

| Strategy          | Generated | Rejected | Notes |
|------------------|----------|----------|-------|
| mutation         |       100 |       ~85 | tamper integrity |
| recombination    |       100 |      100 | chain predecessor |
| specification_gaming |  100 |       ~90 | intent vs formal |
| temporal_drift   |       100 |       ~95 | cumulative drift |
| ...              |          |          |         |

## Investor Pitch

> "APE generates adversarial scenarios modeling known AI failure modes—specification gaming, distribution shift, temporal drift, ambiguity exploitation, and adversarial alignment—and the verifier reports rejection performance by failure class."

This positions APE as a universal AI stress harness, not just a verifier test tool.