# Coh Formal Symbol Reference

## Level Map

| Level | Name | Symbol | Definition |
|-------|------|--------|-----------|
| L0 | Admissibility Law | - | V(y)+S(r)≤V(x)+D(r) |
| L1 | CohBit | β | V(y)+S(r)≤V(x)+D(r)+A(r) |
| L2 | CohAtom | A | V(xn)+∑S≤V(x0)+∑D+∑A |
| L3 | CohState | Σ | Certified state with trace history |
| L4 | CohCategory | CCoh | Morphisms = certified traces |
| L5 | CohField | FCoh | Field over certified states |

## Functional Components

| Component | Symbol | Definition | Condition |
|-----------|--------|------------|----------|
| Spend | S(r) | Execution resource cost | S(r) ≥ 0 |
| Defect | D(r) | Tolerated slack | D(r) ≤ δ |
| Valuation | V(x) | State potential | V(x) ∈ ℝ |
| Authority | A(r) | Budget injection | A(r) ≥ 0 |
| Delta-hat | δ/Δ_max | Max admissible defect | δ ≥ D(r) |

## Canonical Inequalities

### Local (Per-Step)
```
v_post + spend ≤ v_pre + defect + authority
```

### Cumulative (Telescoping)
```
∑ (v_post + spend - v_pre - defect - authority) ≤ 0
```

## File Alignment Matrix

| Rust Implementation | Lean Theorem | Status |
|-----------------|-------------|--------|
| verify_micro.rs | AdmissibilityLaw | ✅ Aligned |
| verify_micro_v3.rs | CohBitTransition | ✅ Aligned |
| verify_chain.rs | CumulativeLaw | ✅ Aligned |
| trajectory/engine.rs | TraceComposition | ✅ Aligned |
| cohbit.rs | CohBitStructure | ✅ Aligned |
| atom.rs | CohAtomMetrics | ✅ Aligned |

## CI Gates

| Check | Command | Status |
|-------|---------|--------|
| Rust Tests | cargo test | ✅ In CI |
| Node Tests | npm test | ✅ In CI |
| Lean Build | lake build | ✅ In CI |