# Verifier Gap Analysis: Paper Theory vs. Rust Implementation

**Generated**: 2026-04-25  
**Purpose**: Identify gaps between academic papers and actual verifier implementation

---

## PDFs Reviewed

| File | Description |
|------|-------------|
| `Coh_Category_Academic.pdf` | Main academic paper on category theory |
| `Coh_Category_Referee.pdf` | Referee response document |
| `Coh core doc/coh-category-zenodo-final/paper/main.tex` | LaTeX source (841 lines) |
| `coh_category_preprint.pdf` | Preprint version |

---

## 1. Core Theory Implemented ✅

| Paper Concept | Rust Implementation | Status |
|--------------|---------------------|--------|
| Verifier (RV) | `coh-node/crates/coh-core/src/verify_micro.rs:11` | ✅ Complete |
| MicroReceipt validation | `verify_micro()` function | ✅ Complete |
| Chain verification | `coh-node/crates/coh-core/src/verify_chain.rs` | ✅ Complete |
| Resource inequality: `v_post + spend ≤ v_pre + defect` | Policy check in `verify_micro` | ✅ Complete |
| Reject codes | `coh-node/crates/coh-core/src/reject.rs` | ✅ Complete |

---

## 2. Semantic Cost / Hidden Layer ⚠️ PARTIAL

| Paper Concept | Rust Implementation | Status |
|--------------|---------------------|--------|
| Hidden traces | `semantic.rs:27` - `HiddenState` | ⚠️ Stub |
| Realizable fiber | `semantic.rs:86` - `RealizableFiber` | ⚠️ Stub |
| Semantic cost computation | `semantic.rs:110` | ⚠️ Stub |
| Projection to observables | `semantic.rs:59` | ⚠️ Stub |

**Gap**: The paper's **Theorem: Subadditive Semantic Cost** (lines 492-499) is not implemented. The Rust has placeholder functions:

```rust
// semantic.rs:139-142 - ALWAYS RETURNS TRUE
pub fn verify_projection_is_certified(...) -> bool {
    !hidden_trace.states.is_empty()  // Placeholder!
}
```

---

## 3. Category Theory Formalization ❌ MISSING

| Paper Concept | Rust Implementation | Status |
|--------------|---------------------|--------|
| Hom-set definition | None | ❌ Not implemented |
| Identity morphism | None | ❌ Not implemented |
| Composition law | None | ❌ Not implemented |
| Category theorem | None | ❌ Not implemented |
| Telescoping law | None | ❌ Not implemented |

The paper defines (main.tex lines 279-383):
- **Hom(x, y)** = set of admissible traces from x to y
- **Identity** = empty trace accepted by verifier  
- **Composition** = trace concatenation with verifier acceptance

The Rust has **no categorical abstraction layer**.

---

## 4. Exact Rational Numerics ⚠️ PARTIAL

| Paper Requirement | Rust Implementation | Status |
|------------------|---------------------|--------|
| ℚ≥0 (non-negative rationals) | u128 integers | ⚠️ Approximation |
| ℚ̄ (extended rationals) | Not implemented | ❌ Missing |
| Rational arithmetic | Integer overflow checking | ⚠️ Partial |
| u128Bounds lemma | u128::try_from | ✅ Implemented |

**Gap**: Paper uses exact rationals; Rust uses fixed-point u128.

---

## 5. Hidden Fibers & Trajectories ⚠️ NOT INTEGRATED

| Paper Concept | Rust Implementation | Status |
|--------------|---------------------|--------|
| Hidden state fibers | `semantic.rs` | ⚠️ Stub only |
| Trajectory search | `trajectory/engine.rs` | ✅ Implemented |
| Admissible trajectories | `AdmissibleTrajectory` type | ✅ Implemented |
| Oplax dissipation constraint | `measurement.rs:64` | ⚠️ Partial |

**Gap**: The paper's **Definition: Hidden Trajectory Cost** and **Axiom: Hidden Cost Subadditivity** are not connected to the trajectory search engine.

---

## 6. Lean Formalization ⚠️ DISCONNECTED

| Component | Status |
|-----------|--------|
| `coh-t-stack/` directory | ❌ Missing from workspace |
| `Coh/Contract/Micro.lean` | ❌ Not found |
| `rv_contract_correctness` theorem | ❌ Not verifiable |
| Lean → Rust traceability matrix | ⚠️ References missing Lean files |

The paper's **Section: Lean 4 Formalization Sketch** (lines 730-841) contains Lean code:
- `axiom rv_id` 
- `axiom rv_comp`
- `def rv`, `def ValidSchema`

**Gap**: These Lean proofs cannot be verified without `coh-t-stack/`.

---

## 7. Summary: Critical Gaps

| Priority | Gap | Impact |
|----------|-----|--------|
| **HIGH** | No Lean formalization present | Cannot verify correctness claims |
| **HIGH** | Semantic cost is stubbed | Cannot compute hidden-cost-aware verification |
| **MEDIUM** | No categorical layer | Cannot prove category theorem |
| **MEDIUM** | Integer vs rational | Potential precision loss |
| **LOW** | Hidden fiber enumeration | Exponential complexity; bounded only |

---

## Recommendations

1. **Restore `coh-t-stack/`** - Lean formalization is referenced but missing
2. **Implement semantic cost** - Critical theorem not operational
3. **Add categorical layer** - Optional but aligns with theory
4. **Document integer approximation** - u128 vs ℚ≥0 tradeoffs

---

## Verified Core Functionality

The verifier correctly implements:
- ✅ Micro receipt schema validation
- ✅ Canon profile hash checking
- ✅ Chain digest linking
- ✅ State hash chaining  
- ✅ Policy violation detection (resource inequality)
- ✅ Numeric overflow protection

The gaps are primarily in the **advanced semantic layer** (hidden traces, fiber enumeration, categorical formalization) rather than the core verification kernel.