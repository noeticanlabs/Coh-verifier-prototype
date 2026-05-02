import Mathlib
import Coh.Physics.Spacetime.SpacetimeTransition

namespace Coh.Physics.Spacetime

/--
## Second Law of Thermodynamics [PROVED]
In an isolated system (Authority = 0, Defect = 0, Spend > 0), 
the valuation V must decrease, meaning entropy increases.
-/
theorem second_law_from_uci
  {Sigma E : Type}
  (sys : SpacetimeTransitionSystem Sigma E ℝ) (σ : Sigma) (e : E) (σ' : Sigma)
  (h_adm : SpacetimeAdmissible sys σ e σ')
  (h_isolated : sys.𝒲 e = 0 ∧ sys.δ e = 0)
  (h_dissipation : 0 < sys.𝒜 e) :
  sys.ℰ σ' < sys.ℰ σ := by
  unfold SpacetimeAdmissible at h_adm
  obtain ⟨_, h_ineq⟩ := h_adm
  obtain ⟨hW, hd⟩ := h_isolated
  rw [hW, hd] at h_ineq
  simp at h_ineq
  -- E' + A <= E
  -- Since A > 0, E' must be strictly less than E
  linarith

/--
## Irreversibility
A transition is irreversible if the valuation strictly decreases.
-/
def Irreversible {Sigma E : Type}
  (sys : SpacetimeTransitionSystem Sigma E ℝ) (σ : Sigma) (e : E) (σ' : Sigma) : Prop :=
  SpacetimeAdmissible sys σ e σ' ∧ sys.ℰ σ' < sys.ℰ σ

end Coh.Physics.Spacetime
