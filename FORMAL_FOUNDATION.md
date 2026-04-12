# Coh Wedge — Formal Foundation

This document specifies the mathematical and logical foundations of the Coh Validator system.

---

## Core Invariant: The Accounting Law of Transitions

For every micro-receipt, the system enforces:

```
v_post + spend <= v_pre + defect
```

Where:
- `v_pre` = unresolved risk/value before the agent step
- `v_post` = unresolved risk/value after the agent step  
- `spend` = operational cost / work consumed in this step
- `defect` = tolerated uncertainty / allowed variance / slack

**Failure** to satisfy this inequality results in a `RejectPolicyViolation` decision.

---

## Lean to Rust Traceability

The core accounting invariant is **formally proved** in Lean 4.

Lean source: [coh-lean/Coh/Core/Chain.lean](coh-lean/Coh/Core/Chain.lean)

---

### The IsLawful Predicate

In `Chain.lean`, the `IsLawful` predicate formalizes the single-step accounting law:

```lean
def IsLawful {V : Type*}
    [NormedAddCommGroup V] [NormedSpace R V] [InnerProductSpace R V] [CarrierSpace V]
    (r : Receipt) (obj obj' : CohObject V) : Prop :=
  obj'.potential obj'.state + r.spend <= obj.potential obj.state + r.defect + r.authority
```

**Rust enforcement**: `crates/coh-core/src/verify_micro.rs`:

```rust
// Constraint: v_post + spend <= v_pre + defect
let lhs = r.metrics.v_post.safe_add(r.metrics.spend)?;
let rhs = r.metrics.v_pre.safe_add(r.metrics.defect)?;
if lhs > rhs { return Reject(RejectPolicyViolation) }
```

---

### The lawful_composition Theorem

In Lean, the composition theorem proves that if every micro-step in a chain is lawful, the aggregate slab is also lawful:

```lean
theorem lawful_composition {V : Type*}
    (r1 r2 : Receipt) (obj1 obj2 obj3 : CohObject V)
    (h1 : IsLawful r1 obj1 obj2)
    (h2 : IsLawful r2 obj2 obj3) :
    IsLawful (combineReceipts r1 r2) obj1 obj3
```

**Rust enforcement**: `verify_slab_envelope` in `crates/coh-core/src/verify_slab.rs`:

```rust
// Macro inequality: v_post_last + total_spend <= v_pre_first + total_defect
let lhs = r.summary.v_post_last.safe_add(r.summary.total_spend)?;
let rhs = r.summary.v_pre_first.safe_add(r.summary.total_defect)?;
if lhs > rhs { return Reject(RejectSlabSummary) }
```

---

## System Layers

### Layer 1: Wire
- All numerical fields encoded as **Decimal Strings**
- JSON format with strict schema enforcement (JCS compatible)

### Layer 2: Runtime
- Converted to `u128` for exact-integer arithmetic
- All arithmetic uses checked operations (`checked_add`, `checked_sub`, `checked_mul`)

### Layer 3: Prehash
- Alphabetized canonical view for deterministic hashing.
- Structurally excludes circular fields.

---

## Determinism Guarantees

The Coh Safety Wedge is designed for absolute execution determinism:
- **No Floating-Point**: Entirely integer-based arithmetic.
- **No Randomness**: Zero reliance on RNG or non-deterministic sources.
- **Canonical Serialization**: Field-alphabetized JSON ensures bit-identical digests.
- **Pure Functions**: Verification logic is isolated from time and external state variability.
