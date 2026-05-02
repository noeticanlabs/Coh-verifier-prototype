import Coh.Boundary.LawOfCoherence
import Coh.Physics.Spacetime.SpacetimeTransition

namespace Coh.Physics.Isomorphism

open Coh.Boundary
open Coh.Physics.Spacetime

/--
## CohBit-Spacetime Isomorphism
A map Φ : X → Σ is an isomorphism if it preserves the transition structure
and the Universal Commit Inequality.
-/
structure Isomorphism
  {X Q Σ E S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S)
  (𝒫 : SpacetimeTransitionSystem Σ E S)
  (Φ : X → Σ)
  (Ψ : Q → E) where

  -- Preservation of Valuation/Energy
  val_pres : ∀ x, 𝒮.V x = 𝒫.ℰ (Φ x)

  -- Preservation of Action/Spend
  spend_pres : ∀ q, 𝒮.Spend q = 𝒫.𝒜 (Ψ q)

  -- Preservation of Defect/Fluctuation
  defect_pres : ∀ q, 𝒮.Defect q = 𝒫.δ (Ψ q)

  -- Preservation of Authority/Work
  auth_pres : ∀ q, 𝒮.Authority q = 𝒫.𝒲 (Ψ q)

  -- Preservation of Admissibility (Physical Law)
  law_pres : ∀ x q y, 𝒮.RV x q y ↔ 𝒫.𝓛_P (Φ x) (Ψ q) (Φ y)

/--
## Canonical Theorem: CohBit-Spacetime Transition Isomorphism
If there exists an isomorphism Φ, then a transition is Coh-admissible
if and only if its image is Spacetime-admissible.
-/
theorem cohbit_spacetime_isomorphism
  {X Q Σ E S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S)
  (𝒫 : SpacetimeTransitionSystem Σ E S)
  (Φ : X → Σ) (Ψ : Q → E)
  (h_iso : Isomorphism 𝒮 𝒫 Φ Ψ) :
  ∀ x q y, CohAdmissible 𝒮 x q y ↔ SpacetimeAdmissible 𝒫 (Φ x) (Ψ q) (Φ y) := by
  intro x q y
  unfold CohAdmissible
  unfold SpacetimeAdmissible
  rw [h_iso.law_pres]
  rw [h_iso.val_pres x]
  rw [h_iso.val_pres y]
  rw [h_iso.spend_pres q]
  rw [h_iso.defect_pres q]
  rw [h_iso.auth_pres q]
  rfl

/--
## Theorem: Isomorphism Preserves the Universal Commit Inequality
[PROVED]

Under an isomorphism that preserves components separately:
- V_C(x) = E_P(Φ x)         -- Valuation/energy
- Spend_C(q) = A_P(Ψ q)   -- Action/dissipation
- Defect_C(q) = δ_P(Ψ q) -- Fluctuation/tolerance
- Authority_C(q) = W_P(Ψ q) -- External work

The commit inequality holds in one system if and only if it holds in the other.

This is the stronger theorem that derives admissibility from the component preservation laws.
-/
theorem isomorphism_preserves_commit_inequality
  {X Q Σ E S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S)
  (𝒫 : SpacetimeTransitionSystem Σ E S)
  (Φ : X → Σ) (Ψ : Q → E)
  (x : X) (q : Q) (y : X)
  -- Component preservation (the beams, bolts, and welds)
  (hV : 𝒮.V x = 𝒫.ℰ (Φ x))
  (hV' : 𝒮.V y = 𝒫.ℰ (Φ y))
  (hSpend : 𝒮.Spend q = 𝒫.𝒜 (Ψ q))
  (hDefect : 𝒮.Defect q = 𝒫.δ (Ψ q))
  (hAuth : 𝒮.Authority q = 𝒫.𝒲 (Ψ q))
  -- The physics inequality (input from mechanics/fluid/etc)
  (h_phys : 𝒫.ℰ (Φ y) + 𝒫.𝒜 (Ψ q) ≤ 𝒫.ℰ (Φ x) + 𝒫.δ (Ψ q) + 𝒫.𝒲 (Ψ q)) :
  -- Derives the CohBit inequality (output)
  𝒮.V y + 𝒮.Spend q ≤ 𝒮.V x + 𝒮.Defect q + 𝒮.Authority q := by
  -- Substitute component equalities
  rw [hV', hSpend, hV, hDefect, hAuth] at h_phys
  -- This is exactly the same inequality in different notation
  exact h_phys

/--
## Theorem: Isomorphism Reflects the Universal Commit Inequality
[PROVED]

The inverse direction: if the CohBit commit inequality holds,
then under the isomorphism, the spacetime inequality holds.
-/
theorem isomorphism_reflects_commit_inequality
  {X Q Σ E S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S)
  (𝒫 : SpacetimeTransitionSystem Σ E S)
  (Φ : X → Σ) (Ψ : Q → E)
  (x : X) (q : Q) (y : X)
  (hV : 𝒮.V x = 𝒫.ℰ (Φ x))
  (hV' : 𝒮.V y = 𝒫.ℰ (Φ y))
  (hSpend : 𝒮.Spend q = 𝒫.𝒜 (Ψ q))
  (hDefect : 𝒮.Defect q = 𝒫.δ (Ψ q))
  (hAuth : 𝒮.Authority q = 𝒫.𝒲 (Ψ q))
  (h_coh : 𝒮.V y + 𝒮.Spend q ≤ 𝒮.V x + 𝒮.Defect q + 𝒮.Authority q) :
  𝒫.ℰ (Φ y) + 𝒫.𝒜 (Ψ q) ≤ 𝒫.ℰ (Φ x) + 𝒫.δ (Ψ q) + 𝒫.𝒲 (Ψ q) := by
  rw [← hV', ← hSpend, ← hV, ← hDefect, ← hAuth] at h_coh
  exact h_coh

/--
## Corollaries: Componentwise Conservation Laws

If Authority = 0 and Spend = 0 and Defect = 0, then energy is conserved.
-/
theorem isomorphism_energy_conservation
  {X Q Σ E S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S)
  (𝒫 : SpacetimeTransitionSystem Σ E S)
  (Φ : X → Σ) (Ψ : Q → E)
  (x : X) (q : Q) (y : X)
  (hV : 𝒮.V x = 𝒫.ℰ (Φ x))
  (hV' : 𝒮.V y = 𝒫.ℰ (Φ y))
  (hSpend : 𝒮.Spend q = 0)
  (hDefect : 𝒮.Defect q = 0)
  (hAuth : 𝒮.Authority q = 0)
  (h_eq : 𝒮.V y + 𝒮.Spend q = 𝒮.V x + 𝒮.Defect q + 𝒮.Authority q) :
  𝒫.ℰ (Φ y) = 𝒫.ℰ (Φ x) := by
  rw [hSpend, hDefect, hAuth, add_zero] at h_eq
  rw [← hV', ← hV] at h_eq
  exact h_eq

end Coh.Physics.Isomorphism
