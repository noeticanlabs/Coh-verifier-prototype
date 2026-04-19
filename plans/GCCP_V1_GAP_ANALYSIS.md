# GCCP v1 to Existing Codebase Mapping

> **Status**: Gap Analysis Complete
> **Purpose**: Map existing Rust/Lean code to GCCP components and identify implementation gaps

---

## Executive Summary

The existing Coh codebase provides a solid foundation for the **Governed Compute Control Plane (GCCP)**, but significant gaps exist between the generic state-transition verification in `coh-core` and the compute-specific control plane specified in GCCP v1. The core accounting law (`v_post + spend ≤ v_pre + defect + authority`) is sound, but the state space, action space, pressure functionals, and compute-specific rejection codes require extension.

**Key Finding**: GCCP is NOT a separate system—it is a **specialization** of the existing Coh object model, adding compute-domain knowledge on top of the receipt-verifiable transition framework.

---

## 1. State Space Mapping (Section 4 vs Existing)

### GCCP Required State (Full 8-tuple)
```
x = (T, P, Q, U, M, R, B, Πc)
  ├── T = (Tdie, Thot, Trise)        # Thermal
  ├── P = (Pnow, Pcap, Pmargin)      # Power
  ├── Q = (qdepth, qage, qmix)        # Queue
  ├── U = (ucore, umem, ulink)        # Utilization
  ├── M = (mused, mbw, mfrag)        # Memory
  ├── R = (rretry, rtimeout, rthrottle)  # Risk/Instability
  ├── B = (bE, bL, bS)                # Budgets (Energy, Latency, Stability)
  └── Πc = (hpolicy, hprofile, mode, class)  # Control Context
```

### Existing Implementation
- **`coh-core/src/types.rs`**: Generic `State` with `id: String`, `value: serde_json::Value`, `hash: String`, `version: u64`
- **`coh-core/src/execute.rs`**: `StateStore` with key-value storage
- **`coh-t-stack/Coh/Kernel/Receipt.lean`**: Receipt has `pre, post, spend, defect, authority` (5 fields)

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| State encoding | Generic JSON Value | Explicit 8-tuple with typed fields | **MAJOR** - No thermal/power/queue decomposition |
| State hashing | SHA-256 on JSON | SHA-256 with domain separation | Partial - canon profile exists |
| Control context | None | Policy hash, profile hash, mode, class | **MAJOR** - Missing |
| Budget tracking | Generic `authority` | Explicit bE, bL, bS | **MAJOR** - Needs specialized budget laws |

### ✅ IMPLEMENTED (Phase 1)
Created at [`coh-node/crates/coh-core/src/gccp/state.rs`](coh-node/crates/coh-core/src/gccp/state.rs).

---

## 2. Action Space Mapping (Section 5 vs Existing)

### GCCP Required Actions
```
A = { adispatch, adefer, areroute, aresize, acooldown, areject }

adispatch  = (k, d, s, β, π)   # kernel, device, stream, batch, priority
adefer     = (k, Δt)           # hold kernel for interval
areroute   = (k, d')          # move to alternate device
aresize    = (k, β')          # shrink/expand batch
acooldown  = (d, Δt)           # thermal recovery window
areject    = (k, c)           # drop with code
```

### Existing Implementation
- **`coh-core/src/execute.rs`**: `Action` struct with `action_type: String`, `target: String`, `params: HashMap<String, serde_json::Value>`, `authority: String`

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| Action typing | Generic String | Typed variants (dispatch, defer, reroute, resize, cooldown, reject) | **MAJOR** - No type-safe actions |
| Dispatch params | Generic JSON | Kernel, device, stream, batch, priority | **MAJOR** - Missing |
| Defer semantics | None | Hold with Δt interval | **MAJOR** - Missing |
| Reroute semantics | None | Move to alternate device | **MAJOR** - Missing |
| Resize semantics | None | Batch size adjustment | **MAJOR** - Missing |
| Cooldown semantics | None | Thermal recovery window | **MAJOR** - Missing |

### ✅ IMPLEMENTED (Phase 1)
Created at [`coh-node/crates/coh-core/src/gccp/actions.rs`](coh-node/crates/coh-core/src/gccp/actions.rs).

---

## 3. Value Functional V(x) Mapping (Section 8 vs Existing)

### GCCP Required V(x) = System Pressure
```
V_p(x) = wT·φT(x) + wP·φP(x) + wQ·φQ(x) + wM·φM(x) + wR·φR(x) + wB·φB(x)

Where:
- φT(x) = λ1·Tdie + λ2·Thot + λ3·(Thot - Tcap)+   # Thermal pressure
- φP(x) = μ1·Pnow + μ2·(Pnow - Pcap)+             # Power pressure
- φQ(x) = ν1·qdepth + ν2·qage + ν3·qmix           # Queue pressure
- φM(x) = ξ1·mused + ξ2·mbw + ξ3·mfrag            # Memory pressure
- φR(x) = ρ1·rretry + ρ2·rtimeout + ρ3·rthrottle  # Instability pressure
- φB(x) = η1/(bE+ε) + η2/(bL+ε) + η3/(bS+ε)       # Budget barrier
```

### Existing Implementation
- **`coh-t-stack/Coh/Kernel/Receipt.lean`**: `pre` and `post` are plain `ℝ` values
- **`coh-core/src/verify_micro.rs`**: Uses `v_pre`, `v_post` as raw numeric strings

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| V computation | None | 6-component weighted sum | **MAJOR** - No pressure functional |
| Thermal term | None | λ1·Tdie + λ2·Thot + λ3·(Thot-Tcap)+ | **MAJOR** - Missing |
| Power term | None | μ1·Pnow + μ2·(Pnow-Pcap)+ | **MAJOR** - Missing |
| Queue term | None | ν1·qdepth + ν2·qage + ν3·qmix | **MAJOR** - Missing |
| Memory term | None | ξ1·mused + ξ2·mbw + ξ3·mfrag | **MAJOR** - Missing |
| Risk term | None | ρ1·rretry + ρ2·rtimeout + ρ3·rthrottle | **MAJOR** - Missing |
| Budget barrier | None | η/(b+ε) inverse barrier | **MAJOR** - Missing |
| Policy index | None | Weights indexed by policy p | **MAJOR** - Missing |

### Recommendation
Implement pressure functional module:

```rust
// NEW: coh-core/src/gccp/pressure.rs
pub struct PressureConfig {
    pub wT: f64, pub wP: f64, pub wQ: f64, pub wM: f64, pub wR: f64, pub wB: f64,
    // thermal coefficients
    pub lambda1: f64, pub lambda2: f64, pub lambda3: f64,
    // power coefficients
    pub mu1: f64, pub mu2: f64,
    // ... etc
}

pub fn compute_pressure(state: &GccpState, config: &PressureConfig, policy: &Policy) -> u128;
```

---

## 4. Spend/Defect Functionals Mapping (Sections 9-10 vs Existing)

### GCCP Required Spend
```
Spend(x,a,ẋ',r) = αE·E(a,x) + αT·ΔT(a,x) + αQ·ΔQ+(a,x) + αL·L(a,x) + αW·W(a,x)

Where:
- E(a,x)    = predicted joules
- ΔT(a,x)   = predicted thermal rise  
- ΔQ+(a,x)  = max(q'depth - qdepth, 0)
- L(a,x)    = latency budget consumed
- W(a,x)    = wear / stress proxy
```

### GCCP Required Defect
```
Defect*(x,a,ẋ',r) = dpred + dtele + dmodel + dquant

Where:
- dpred   = predictor error bound
- dtele   = telemetry staleness / sensor uncertainty
- dmodel  = coarse model mismatch
- dquant  = fixed-point / interval widening slack
```

### Existing Implementation
- **`coh-t-stack/Coh/Kernel/Receipt.lean`**: `spend: ℝ`, `defect: ℝ` as plain reals
- **`coh-core/src/measurement.rs`**: `step_cost(r) = spend + defect`

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| Spend composition | Single value | 5-component weighted sum | **MAJOR** - No decomposition |
| Energy term | None | αE·E(a,x) | **MAJOR** - Missing |
| Thermal delta | None | αT·ΔT(a,x) | **MAJOR** - Missing |
| Queue delta+ | None | αQ·ΔQ+(a,x) | **MAJOR** - Missing |
| Latency term | None | αL·L(a,x) | **MAJOR** - Missing |
| Wear term | None | αW·W(a,x) | **MAJOR** - Missing |
| Defect composition | Single value | 4-component sum | **MAJOR** - No decomposition |
| Predictor bound | None | dpred | **MAJOR** - Missing |
| Telemetry uncertainty | None | dtele | **MAJOR** - Missing |
| Model mismatch | None | dmodel | **MAJOR** - Missing |
| Quantization slack | None | dquant | **MAJOR** - Missing |

---

## 5. Hard Guard Mapping (Section 11 vs Existing)

### GCCP Required Hard Guards
```
G_p(x,a,ẋ',r) ⟺
  Ẏhot ≤ Tmax(p)
  ∧ Ÿnow ≤ Pmax(p)
  ∧ q̂age ≤ Qmax(p)
  ∧ m̂used ≤ Mmax(p)
  ∧ r̂throttle ≤ Rmax(p)
  ∧ Defect*_p(x,a,ẋ',r) ≤ δp
  ∧ bE ≥ Emin(p) ∧ bL ≥ Lmin(p) ∧ bS ≥ Smin(p)
```

### Existing Implementation
- Partial in `coh-core/src/verify_micro.rs`: Checks `spend ≤ v_pre` (one domain constraint)
- No explicit temperature, power, queue, memory caps

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| Thermal cap | None | Ÿhot ≤ Tmax(p) | **MAJOR** - Missing |
| Power cap | None | Ÿnow ≤ Pmax(p) | **MAJOR** - Missing |
| Queue age cap | None | q̂age ≤ Qmax(p) | **MAJOR** - Missing |
| Memory cap | None | m̂used ≤ Mmax(p) | **MAJOR** - Missing |
| Throttle cap | None | r̂throttle ≤ Rmax(p) | **MAJOR** - Missing |
| Defect cap | None | Defect* ≤ δp | **MAJOR** - Missing |
| Budget mins | None | bE ≥ Emin, bL ≥ Lmin, bS ≥ Smin | **MAJOR** - Missing |

---

## 6. Admissibility Predicate Mapping (Section 13 vs Existing)

### GCCP Required
```
Adm_p(x,a,ẋ',r) ⟺
  a ∈ A(x)               # action available in state
  ∧ ẋ' = Π(x,a)          # predict successor
  ∧ G_p(x,a,ẋ',r)        # hard guards hold
  ∧ (V(ẋ') + Spend ≤ V(x) + Defect)  # governing inequality
```

### Existing Implementation
- **`coh-t-stack/Coh/Kernel/Verifier.lean`**: 
  ```lean
  def Lawful (r : Receipt) : Prop :=
    r.post + r.spend <= r.pre + r.defect + r.authority ∧ r.spend <= r.pre
  ```
- **`coh-core/src/verify_micro.rs`**: Checks `v_post + spend ≤ v_pre + defect + authority`

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| Action availability | None | a ∈ A(x) check | **MAJOR** - Missing |
| Predictor Π | None | ẋ' = Π(x,a) | **MAJOR** - No predictor |
| Hard guards G | None | G_p check before inequality | **MAJOR** - Not separate |
| Full inequality | v_post+spend ≤ v_pre+defect+auth | V(ẋ')+Spend ≤ V(x)+Defect | **MINOR** - V vs raw values |

---

## 7. Decision Type Mapping (Section 14 vs Existing)

### GCCP Required
```
D = {ACCEPT, REJECT, DEFER, REROUTE}
```

### Existing Implementation
- **`coh-core/src/types.rs`**:
  ```rust
  pub enum Decision {
      Accept,
      Reject,
      SlabBuilt,
      TerminalSuccess,
      TerminalFailure,
      AbortBudget,
  }
  ```

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| ACCEPT | ✅ Accept | ✅ Same | **NONE** |
| REJECT | ✅ Reject | ✅ Same | **NONE** |
| DEFER | ❌ Missing | ✅ Hold and retry later | **MAJOR** - No defer semantics |
| REROUTE | ❌ Missing | ✅ Move to alternate device | **MAJOR** - No reroute semantics |
| SlabBuilt | ❌ Not in GCCP | ❌ N/A | Extra |
| TerminalSuccess | ❌ Not in GCCP | ❌ N/A | Extra |
| TerminalFailure | ❌ Not in GCCP | ❌ N/A | Extra |
| AbortBudget | ❌ Not in GCCP | ❌ N/A | Extra |

### Recommendation
Extend Decision enum:

```rust
pub enum Decision {
    Accept,
    Reject,
    Defer,       // NEW: hold and retry
    Reroute,     // NEW: move to alternate
    SlabBuilt,   // Keep for slab receipts
    // ... other variants can be deprecated
}
```

---

## 8. Reject Code Mapping (Section 18 vs Existing)

### GCCP Required Compute-Specific Codes
```
REJECT_TEMP_CAP, REJECT_POWER_CAP, REJECT_QUEUE_CAP, 
REJECT_MEMORY_CAP, REJECT_DEFECT_CAP, REJECT_BUDGET,
REJECT_PREDICTOR_STALE, REJECT_TELEMETRY_STALE,
REJECT_ROUTE_UNAVAILABLE, REJECT_POLICY_CLASS_MISMATCH
```

### Existing Implementation
- **`coh-core/src/reject.rs`**: 21 codes
- **`coh-t-stack/Coh/Contract/RejectCode.lean`**: 21 codes (same set)

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| REJECT_TEMP_CAP | ❌ Missing | ✅ | **MAJOR** |
| REJECT_POWER_CAP | ❌ Missing | ✅ | **MAJOR** |
| REJECT_QUEUE_CAP | ❌ Missing | ✅ | **MAJOR** |
| REJECT_MEMORY_CAP | ❌ Missing | ✅ | **MAJOR** |
| REJECT_DEFECT_CAP | ❌ Missing | ✅ | **MAJOR** |
| REJECT_BUDGET | ❌ Missing | ✅ | **MAJOR** |
| REJECT_PREDICTOR_STALE | ❌ Missing | ✅ | **MAJOR** |
| REJECT_TELEMETRY_STALE | ❌ Missing | ✅ | **MAJOR** |
| REJECT_ROUTE_UNAVAILABLE | ❌ Missing | ✅ | **MAJOR** |
| REJECT_POLICY_CLASS_MISMATCH | ❌ Missing | ✅ | **MAJOR** |

### Recommendation
Add compute-specific codes:

```rust
// Add to coh-core/src/reject.rs
pub enum RejectCode {
    // ... existing codes ...
    
    // GCCP compute-specific (NEW)
    RejectTempCap,
    RejectPowerCap,
    RejectQueueCap,
    RejectMemoryCap,
    RejectDefectCap,
    RejectBudget,
    RejectPredictorStale,
    RejectTelemetryStale,
    RejectRouteUnavailable,
    RejectPolicyClassMismatch,
}
```

---

## 9. Proof Obligations Mapping (Section 28 vs Existing)

### GCCP Required Proofs
1. **Determinism**: Same canonical inputs → Same verifier decision
2. **Closure**: Accepted receipt chains → Chain digest + state hash linkage holds
3. **Soundness**: ACCEPT → Guards + governing inequality hold
4. **Budget boundedness**: bE,k, bL,k, bS,k ≥ 0 for all accepted steps
5. **Refinement monotonicity**: p1 ⊑ p2 → admissible pairs subset
6. **Strict throttle theorem**: p_T ⊏ p_N (throttled strictly refines normal)

### Existing Implementation
- **`coh-t-stack/Coh/Kernel/Verifier.lean`**:
  - `verify_sound`: verify(r) = accept → Lawful r
  - `verify_complete`: Lawful r → verify(r) = accept
- **`coh-t-stack/Coh/Core/Law.lean`**: Basic lawfulness theorems

### Gap Analysis

| Proof | Existing | GCCP Required | Gap |
|-------|----------|---------------|-----|
| Determinism | Partial | Full canonical input determinism | **MINOR** - Needs formalization |
| Closure | Partial | Receipt chains → digest continuity | **MINOR** |
| Soundness | ✅ verify_sound | + hard guards + inequality | Needs extension |
| Budget boundedness | None | bE,k, bL,k, bS,k ≥ 0 | **MAJOR** - No budget tracking |
| Refinement monotonicity | None | Policy lattice properties | **MAJOR** - No policy refinement |
| Strict throttle theorem | None | p_T ⊏ p_N witness | **MAJOR** - No theorem |

---

## 10. Test Vector Mapping (Section 29 vs Existing)

### GCCP Required Vectors
```
Base Coh vectors:
- accept-minimal
- bad schema
- bad chain digest
- invalid interval
- overflow
- valid slab Merkle
- bad slab summary

GCCP-specific vectors:
- reject on temp-cap breach
- reject on power-cap breach
- reject on stale telemetry
- reject on stale predictor
- defer on high queue pressure with valid later slot
- reroute on alternate admissible device
```

### Existing Implementation
- **`coh-node/vectors/adversarial/`**: 6 rejection test files + 1 valid
- **`coh-node/vectors/valid/`**: Valid chains of 10, 100, 1000 steps

### Gap Analysis

| Vector | Existing | GCCP Required | Gap |
|--------|----------|---------------|-----|
| accept-minimal | Partial | Same | **MINOR** |
| bad schema | ✅ reject_schema.jsonl | Same | **NONE** |
| bad chain digest | ✅ reject_chain_digest.jsonl | Same | **NONE** |
| invalid interval | ✅ reject_interval_invalid (in code) | Same | **NONE** |
| overflow | ✅ reject_overflow.jsonl | Same | **NONE** |
| valid slab Merkle | ✅ (slab verification exists) | Same | **NONE** |
| bad slab summary | ✅ (slab verification exists) | Same | **NONE** |
| temp-cap breach | ❌ Missing | ✅ | **MAJOR** |
| power-cap breach | ❌ Missing | ✅ | **MAJOR** |
| stale telemetry | ❌ Missing | ✅ | **MAJOR** |
| stale predictor | ❌ Missing | ✅ | **MAJOR** |
| defer on queue | ❌ Missing | ✅ | **MAJOR** |
| reroute on alternate | ❌ Missing | ✅ | **MAJOR** |

---

## 11. Morphism/Measurement Mapping (Section 26 vs Existing)

### GCCP Required Morphism
```
f = (fX, fR, Δf)
  fX: X_A → X_B    # state mapping
  fR: R_A → R_B    # receipt mapping  
  Δf: slack        # additive slack

Lawfulness preservation:
  RV_A(x,r,x') = ACCEPT → RV_B(fX(x), fR(r), fX(x')) = ACCEPT
  
Oplax inequality:
  V_B(fX(x')) + Spend_B(fR(r)) ≤ V_B(fX(x)) + Defect_B(fR(r)) + Δf
```

### Existing Implementation
- **`coh-core/src/measurement.rs`**: `Measurement` trait with `map_step`, `verify_chain_dissipation`
- **`coh-t-stack/Coh/Oplax/Slack.lean`**: `OplaxMorphism` with `slack: ℝ`
- **`coh-t-stack/Coh/Oplax/Morphism.lean`**: `StrictMorphism`, composition theorems

### Gap Analysis

| Component | Existing | GCCP Required | Gap |
|-----------|----------|---------------|-----|
| State mapping fX | Generic | Compute state mapping | Needs specialization |
| Receipt mapping fR | Generic | GCCP receipt mapping | Needs specialization |
| Slack Δf | ✅ Exists | ✅ Same | **NONE** |
| Dissipation verification | ✅ verify_chain_dissipation | V + Spend inequality | **MINOR** - V vs raw |
| Cross-device morphisms | None | Device class A → B | **MAJOR** - Missing |

---

## 12. Minimal Concrete Instantiation (Section 31)

GCCP Section 31 specifies a minimal implementation for first deployment:

```
x = (T, E, Q)    # Reduced state: Thermal, Energy, Queue

Actions: {idle, light, medium, heavy, cool}

V(x) = linear pressure functional
Action deltas = deterministic
Guard caps = explicit

Proof: heavy action admissible under Normal, inadmissible under Throttled
```

### Gap Analysis

| Component | Required | Status |
|-----------|----------|--------|
| Reduced state (T,E,Q) | Required | **MAJOR** - Not implemented |
| 5 discrete actions | Required | **MAJOR** - Not implemented |
| Linear pressure | Required | **MAJOR** - Not implemented |
| Deterministic deltas | Required | **MAJOR** - Not implemented |
| Guard caps | Required | **MAJOR** - Not implemented |
| Throttle theorem proof | Required | **MAJOR** - Not implemented |

---

## ✅ IMPLEMENTATION COMPLETE

As of 2026-04-19, Phases 1-3 have been fully implemented with 53 passing tests.

### Files Created
- `coh-node/crates/coh-core/src/gccp/state.rs` - State types (Thermal, Power, Queue, Memory, Risk, Budgets, Context)
- `coh-node/crates/coh-core/src/gccp/actions.rs` - Action types (Dispatch, Defer, Reroute, Resize, Cooldown, Reject)
- `coh-node/crates/coh-core/src/gccp/pressure.rs` - V(x) functional with 6 components
- `coh-node/crates/coh-core/src/gccp/spend.rs` - Spend/Defect functionals
- `coh-node/crates/coh-core/src/gccp/guards.rs` - Hard guard predicates
- `coh-node/crates/coh-core/src/gccp/predictor.rs` - State predictor Π(x,a)
- `coh-node/crates/coh-core/src/gccp/admissibility.rs` - Admissibility predicate
- `coh-node/crates/coh-core/src/gccp/mod.rs` - Module root

### Previously Critical Gaps (Now Fixed)
- Decision::Defer, Decision::Reroute - ADDED
- 10 compute-specific RejectCodes - ADDED
- GccpState 8-tuple - ADDED
- GccpAction 6 variants - ADDED
- Pressure V(x) 6 components - ADDED
- Spend/Defect decomposition - ADDED
- Hard guards - ADDED
- Predictor Π - ADDED
- Admissibility predicate - ADDED

---

## Summary: Gap Prioritization

### Critical Gaps (Must Fix)

1. **Decision Types**: Add DEFER, REROUTE
2. **Reject Codes**: Add 10 compute-specific codes
3. **State Space**: Create GccpState with thermal, power, queue, memory, risk, budgets, context
4. **Action Space**: Create typed GccpAction enum
5. **Pressure Functional**: Implement V(x) with 6 components
6. **Spend/Defect**: Decompose into compute-specific terms
7. **Hard Guards**: Implement thermal, power, queue, memory, defect, budget caps
8. **Predictor**: Implement Π(x,a) for state prediction
9. **Admissibility**: Implement full Adm predicate with availability, predictor, guards, inequality

### Important Gaps (Should Fix)

10. **Budget Tracking**: Implement bE, bL, bS with boundedness proofs
11. **Policy Lattice**: Implement refinement relation and theorems
12. **Test Vectors**: Add 6 GCCP-specific vectors
13. **Cross-device Morphisms**: Implement device class mappings

### Minor Gaps (Nice to Have)

14. **Strict Throttle Theorem**: Formal proof in Lean
15. **Determinism Formalization**: Full canonical determinism proof
16. **Positive Variation Accounting**: Implement Σ max(Δ, 0) for queue/thermal

---

## Implementation Strategy

Given the gap analysis, the recommended implementation order is:

```mermaid
flowchart LR
    A[Phase 1: Core Types] --> B[Phase 2: Pressure & Spend]
    B --> C[Phase 3: Verifier Extension]
    C --> D[Phase 4: Decision Logic]
    D --> E[Phase 5: Test Vectors]
    E --> F[Phase 6: Proofs]
    
    A1[GccpState] --> A
    A2[GccpAction] --> A
    A3[RejectCodes] --> A
    
    B1[Pressure Functional] --> B
    B2[Spend/Defect Terms] --> B
    B3[Hard Guards] --> B
    
    C1[Predictor] --> C
    C2[Adm Predicate] --> C
    C3[Budget Laws] --> C
    
    D1[DEFER/REROUTE] --> D
    D2[Scheduler Hooks] --> D
    
    E1[Cap Breach Tests] --> E
    E2[Defer/Reroute Tests] --> E
    
    F1[Boundedness] --> F
    F2[Throttle Theorem] --> F
```

---

## Files to Create/Modify

### New Files (Phase 1-3)
- `coh-node/crates/coh-core/src/gccp/mod.rs` - GCCP module root
- `coh-node/crates/coh-core/src/gccp/state.rs` - GccpState types
- `coh-node/crates/coh-core/src/gccp/actions.rs` - GccpAction types
- `coh-node/crates/coh-core/src/gccp/pressure.rs` - Pressure functional
- `coh-node/crates/coh-core/src/gccp/spend.rs` - Spend/Defect computation
- `coh-node/crates/coh-core/src/gccp/guards.rs` - Hard guards
- `coh-node/crates/coh-core/src/gccp/predictor.rs` - State predictor
- `coh-node/crates/coh-core/src/gccp/admissibility.rs` - Adm predicate
- `coh-node/crates/coh-core/src/gccp/verifier.rs` - GCCP verifier

### Files to Modify (Phase 1-4)
- `coh-node/crates/coh-core/src/types.rs` - Add DEFER, REROUTE
- `coh-node/crates/coh-core/src/reject.rs` - Add 10 compute codes
- `coh-node/crates/coh-core/src/verify_micro.rs` - Extend with GCCP checks

### Lean Files (Phase 6)
- `coh-t-stack/Coh/GCCP/Object.lean` - GCCP object artifact
- `coh-t-stack/Coh/GCCP/State.lean` - GccpState type
- `coh-t-stack/Coh/GCCP/Pressure.lean` - Pressure functional
- `coh-t-stack/Coh/GCCP/ThrottleTheorem.lean` - Strict throttle proof

---

## Conclusion

The existing Coh codebase provides the **foundational framework** (receipt-verifiable transitions, the accounting law, canonicalization, chain digestion, Merkle aggregation), but GCCP v1 requires **significant specialization** for compute control. The gap is NOT in the core verification kernel—it is in the **domain-specific knowledge** (thermal/power/queue pressure, compute actions, hardware-specific guards).

**The implementation strategy is to build GCCP as a layer on top of coh-core, not as a replacement.**