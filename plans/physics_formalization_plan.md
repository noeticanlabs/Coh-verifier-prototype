# Coh Physics Formalization Plan

## Current State

### Lean Modules (coh-t-stack)
```
Coh.Physics.Spinor/
  Basic.lean      - Spinor type definition
  Gamma.lean     - Gamma matrices (γ⁰, γ¹, γ², γ³)
  Current.lean   - Dirac current Jμ
  Proofs.lean    - Theorems

Coh.Boundary/
  LorentzGmiSmooth.lean - GMI barrier potential
  CohSpinor.lean        - Coh spinor definition
```

### Rust Physics (coh-physics crate)
```rust
src/
  current.rs    - CoherenceCurrent Jμ computation
  gamma.rs      - Gamma matrices γ⁰,γ¹,γ²,γ³
  measurement.rs - SpinorProjector Πⁱ
  proofs.rs     - verify_gamma0_sq_eq_identity()
```

## Integration Architecture

```
┌─────────────────────────────────────────────────────┐
│  Lean 4 (coh-t-stack)                               │
│  ┌─────────────────────────────────────────────┐    │
│  │ Coh.Physics.Spinor                          │    │
│  │   - spinor : CohSpinor                      │    │
│  │   - gamma (γ⁰,γ¹,γ²,γ³)                    │    │
│  │   - current (Jμ)                           │    │
│  │   THEOREMS: density ≥ 0, γ² = I, etc.      │    │
│  └─────────────────────────────────────────────┘    │
│                        ↑ FFI                        │
│  ┌─────────────────────────────────────────────┐    │
│  │ kernel_invariants.rs                        │    │
│  │   coh_check_positive_density(...)          │    │
│  │   assert_spinor_invariants(...)            │    │
│  └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│  Rust (coh-node)                                    │
│  ┌─────────────────────────────────────────────┐    │
│  │ coh-physics                                 │    │
│  │   - CohSpinor { components: [4]Complex64 }  │    │
│  │   - gamma0(), gamma1(), ...                 │    │
│  │   - CoherenceCurrent::new()                 │    │
│  │   - SpinorProjector                          │    │
│  └─────────────────────────────────────────────┘    │
│                        ↑                            │
│  ┌─────────────────────────────────────────────┐    │
│  │ coh-genesis                                 │    │
│  │   - GmiAtom                                │    │
│  │   - verify_cohbit()                        │    │
│  └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

## Formalization Goals

### Phase 1: Core Spinor Physics
1. **Spinor Type** - Define `CohSpinor` in Lean matching Rust
2. **Gamma Matrices** - Prove γ² = I, {γᵐ,γⁿ} = 2gᵐⁿ
3. **Density** - Prove `density ψ ≥ 0` (positive_density_theorem)
4. **Normalization** - Prove `‖ψ‖ = 1` (cohspinor_density_eq_one)

### Phase 2: Current & Conservation
5. **Current** - Define Jμ = ψ̄γ⁰γᵐψ (conserved current)
6. **Continuity** - ∂μJμ = 0 (current conservation)

### Phase 3: Integrability
7. **GMI** - Relate spinor density to viability clock
8. **Boundary** - Link to CohBit/CohAtom states

## Key Theorems to Formalize

| Lean Theorem | Rust Invariant | Description |
|-------------|----------------|--------------|
| `positive_density_theorem` | `spinor_density_nonneg` | ψ̄ψ ≥ 0 always |
| `gamma0_sq_eq_one` | `gamma0_sq_eq_identity` | γ⁰² = I₄ |
| `gamma_trace_zero` | `gamma_trace_is_zero` | tr(γᵐ) = 0 |
| `current_conservation` | `j_divergence_zero` | ∂μJμ = 0 |
| `gmi_viability` | `viability_from_density` | v ≤ c from density |

## Implementation Strategy

1. **Mirror First** - Write Lean from existing Rust signatures
2. **Prove Incremental** - Start with algebraic identities
3. **FFI Integration** - Replace stubs with Lean calls
4. **Export for Runtime** - Compile to `Coh.a` static lib

## Files to Create

```
coh-t-stack/Coh/Physics/Spinor/
├── Basic.lean          (COMPLETE - Spinor type)
├── Gamma.lean         (IN PROGRESS - Gamma matrices)
├── Current.lean       (TODO - Jμ definition)
├── Proofs.lean        (TODO - theorems)
└── FFI.lean          (TODO - FFI exports)

coh-t-stack/Coh/Boundary/
├── LorentzGmiSmooth.lean  (COMPLETE - barrier potential)
└── Integration.lean       (TODO - spinor↔boundary)
```

## Next Steps

1. Add missing imports to existing Lean files
2. Complete `Gamma.lean` with all 4 matrices
3. Define `Current.lean` with Jμ
4. Prove `positive_density_theorem`
5. Create FFI stub in `kernel_invariants.rs`