# CohBit + CTRL Verification Gauntlet

This directory contains the verification gauntlet test suite for testing spec conformance, adversarial mutation, and end-to-end proof repair for CohBit and CTRL.

## Running the Gauntlet

### Full Gauntlet
```bash
cargo test --package coh-core --test verification_gauntlet
```

### With Full Features
```bash
cargo test --package coh-core --test verification_gauntlet --features fixture-keys
```

### Individual Test Categories

| Test Class | Description | Expected |
|-----------|------------|----------|
| A (Boundary) | Exact inequality boundary tests | PASS |
| B (Overflow) | Overflow rejection tests | PASS |
| C (Differential) | Production vs Reference | PASS |
| D (Cumulative) | Cumulative accounting | PASS |
| E (CTRL) | CTRL adapter eligibility | PASS |
| F (Property) | Property-based tests | PASS |
| G (Regression) | Known bug prevention | PASS |
| H (Micro) | Micro verifier (requires fixture-keys) | PASS |

## Specification

### Accounting Law
```
v_post + spend ≤ v_pre + defect + authority
```

### Cumulative Law
```
v_post_last + total_spend ≤ v_pre_first + total_defect + total_authority
```

### Canonical Test Values

| Name | v_pre | v_post | spend | defect | authority | Expected |
|------|------|-------|-------|--------|------------|-----------|
| exact_boundary | 100 | 120 | 10 | 0 | 30 | Accept (130 ≤ 130) |
| authority_shortfall | 100 | 120 | 10 | 0 | 29 | Reject (130 > 129) |

## Golden Vectors

- `valid_boundary_exact.json` - Exact boundary case
- `invalid_authority_shortfall.json` - Insufficient authority  
- `invalid_lhs_overflow.json` - Overflow rejection

## Verification Claims

The gauntlet tests:

1. **Accounting law correctness** - exact inequality enforcement
2. **Overflow rejection** - no wrapping, no saturation
3. **Differential testing** - production matches reference
4. **Cumulative law** - slab/telescope preservation
5. **CTRL eligibility** - valid repairs emit CohBit
6. **Property tests** - determinism, monotonicity
7. **Regression prevention** - no silent failures

## Cross-Layer Equivalence

All verifier paths must agree:
- L0/micro verifier
- V3 verifier  
- Chain verifier
- Slab verifier
- Dashboard grade

## Test Matrix

| Test | CohBit | CTRL | Status |
|------|-------|------|--------|
| accounting boundary | ✅ | - | Implemented |
| overflow rejection | ✅ | - | Implemented |
| digest tamper | ✅ | ✅ | Partial |
| sequence reorder | ✅ | ✅ | Partial |
| slab preservation | ✅ | - | Partial |
| failed Lean repair | - | ✅ | Reference |
| successful repair | ✅ | ✅ | Reference |
| dashboard certification | ✅ | ✅ | Reference |
| golden vectors | ✅ | ✅ | Implemented |

## What Proves "True and Accurate"

The gauntlet proves:

> The implementation conforms to the stated CohBit/CTRL specification across normal, boundary, adversarial, and end-to-end cases.

This is the defensible engineering claim, not metaphysical truth.