import Mathlib
import Coh.Physics.Spacetime.SpacetimeTransition

namespace Coh.Physics.Spacetime

/--
## Conservation of Energy
If there is zero dissipation (𝒜=0), zero fluctuation (δ=0), and zero external work (𝒲=0),
then energy is conserved: ℰ[Σ'] ≤ ℰ[Σ].
-/
theorem energy_conservation_from_inequality {Sigma E S : Type} [OrderedAddCommMonoid S]
  (sys : SpacetimeTransitionSystem Sigma E S) (σ : Sigma) (e : E) (σ' : Sigma)
  (h_adm : SpacetimeAdmissible sys σ e σ')
  (h_diss : sys.𝒜 e = 0)
  (h_fluc : sys.δ e = 0)
  (h_work : sys.𝒲 e = 0) :
  sys.ℰ σ' ≤ sys.ℰ σ := by
  unfold SpacetimeAdmissible at h_adm
  obtain ⟨_, h_ineq⟩ := h_adm
  rw [h_diss, h_fluc, h_work] at h_ineq
  simp at h_ineq
  exact h_ineq

/--
## Conservative Flow
A physical evolution is conservative if energy is strictly preserved.
This is a special case of the UCI where equality holds.
-/
def IsConservative {Sigma E S : Type} [OrderedAddCommMonoid S]
  (sys : SpacetimeTransitionSystem Sigma E S) (σ : Sigma) (e : E) (σ' : Sigma) : Prop :=
  SpacetimeAdmissible sys σ e σ' ∧ sys.ℰ σ' = sys.ℰ σ

end Coh.Physics.Spacetime
