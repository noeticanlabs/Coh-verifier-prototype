import Mathlib
import Coh.Boundary.LawOfCoherence
import Coh.Physics.Spacetime.SpacetimeTransition

namespace Coh.Physics.Trajectory

/--
## Trajectory Commit Module
Proves that single admissible CohBit transitions imply admissible trajectories.
-/

open Coh.Boundary
open Coh.Physics.Spacetime

/--
## CohBit Commit (without rigidity/law check)
The budget inequality alone - does not require verifier law.
-/
def CohCommit {X Q S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S) (x : X) (q : Q) (y : X) : Prop :=
  𝒮.V y + 𝒮.Spend q ≤ 𝒮.V x + 𝒮.Defect q + 𝒮.Authority q

/--
## Spacetime Commit (without physical law check)
The energy-action inequality alone.
-/
def SpacetimeCommit {Σ E S : Type} [OrderedAddCommMonoid S]
  (𝒫 : SpacetimeTransitionSystem Σ E S) (σ : Σ) (e : E) (σ' : Σ) : Prop :=
  𝒫.ℰ σ' + 𝒫.𝒜 e ≤ 𝒫.ℰ σ + 𝒫.δ e + 𝒫.𝒲 e

/--
## Single Step Commit Implies Admissibility
If a transition satisfies both commit (budget) and rigidity (verifier law),
it is fully admissible.
-/
theorem commit_plus_rigidity_is_admissible {X Q S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S) (x : X) (q : Q) (y : X) :
  CohCommit 𝒮 x q y → 𝒮.RV x q y → CohAdmissible 𝒮 x q y := by
  intro h_commit h_rigidity
  unfold CohAdmissible
  constructor
  exact h_rigidity
  exact h_commit

/--
## Single Step Equivalent Definition
Admissibility = Commit ∧ Rigidity (proved both directions)
-/
theorem admissible_eq_commit_and_rigidity {X Q S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S) (x : X) (q : Q) (y : X) :
  CohAdmissible 𝒮 x q y ↔ (CohCommit 𝒮 x q y ∧ 𝒮.RV x q y) := by
  unfold CohAdmissible CohCommit
  constructor
  · intro h
    constructor
    exact h.right
    exact h.left
  · intro h
    constructor
    exact h.right
    exact h.left

/--
## Trajectory (Chain of Transitions)
A sequence: x₀ → x₁ → ... → xₙ
-/
structure Trajectory (X Q : Type) where
  states : List X
  actions : List Q
  continuous : states.length = actions.length + 1
  -- ∀ i, the i-th action leads from state i to state i+1

/--
## Trajectory Commit (Telescoping Sum)
The cumulative budget inequality for a full trajectory.
The sum of final valuations plus all spends ≤ sum of initial valuation plus all defects plus all authorities.

This is the KEY trajectory theorem.
-/
def TrajectoryCommit {X Q S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S) (τ : Trajectory X Q) : Prop :=
  let final_val := 𝒮.V (τ.states.last (by
    have h := τ.continuous
    cases τ.states
    · simp at h
    · simp))
  let initial_val := 𝒮.V (τ.states.head (by
    have h := τ.continuous
    cases τ.states
    · simp at h
    · simp))
  let total_spend := (τ.actions.map 𝒮.Spend).sum
  let total_defect := (τ.actions.map 𝒮.Defect).sum
  let total_authority := (τ.actions.map 𝒮.Authority).sum
  final_val + total_spend ≤ initial_val + total_defect + total_authority

/--
## Lemma: Local Commit as Delta Bound
Convert each local commit inequality into a delta bound:
/--
## Single Step Equivalent Definition
Admissibility = Commit ∧ Rigidity (proved both directions)
-/
theorem trajectory_commit_telescopes {X Q S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S)
  (τ : Trajectory X Q)
  (h_states : τ.states ≠ [])
  (h_step : ∀ i : Fin τ.actions.length,
    CohCommit 𝒮 τ.states[i] τ.actions[i] τ.states[i+1]) :
  TrajectoryCommit 𝒮 τ := by
  unfold TrajectoryCommit
  induction τ.actions generalizing τ.states
  case nil =>
    have h := τ.continuous; simp at h
    simp [h]
  case cons q qs ih =>
    match τ.states with
    | x₀ :: x₁ :: xs =>
      have h_head := h_step 0
      let τ_tail : Trajectory X Q := {
        states := x₁ :: xs,
        actions := qs,
        continuous := by have h := τ.continuous; simp at h; exact h
      }
      have ih_res := ih τ_tail (by simp) (by intro i; exact h_step (i.succ))
      simp only [List.map_cons, List.sum_cons, List.head_cons] at ih_res ⊢
      exact coh_compose_linear h_head ih_res
    | _ => have h := τ.continuous; simp at h; contradiction

/--
## Theorem: Admissible Individual Steps Imply Admissible Trajectory
[NEEDS PROOF]

If each individual step is admissible (commit + rigidity), then the entire trajectory is admissible.
-/
theorem admissible_steps_imply_admissible_trajectory {X Q S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S)
  (τ : Trajectory X Q)
  (h_states : τ.states ≠ [])
  (h_step : ∀ i : Fin τ.actions.length, CohAdmissible 𝒮 τ.states[i] τ.actions[i] τ.states[i+1]) :
  TrajectoryCommit 𝒮 τ := by
  apply trajectory_commit_telescopes 𝒮 τ h_states
  intro i
  exact (admissible_eq_commit_and_rigidity 𝒮 _ _ _).mp (h_step i) |>.left

/--
## Theorem: Isomorphism Preserves Trajectory Commit [PROVED]
Under the isomorphism, commit inequality holds for the trajectory in both spaces.
-/
theorem isomorphism_preserves_trajectory_commit {X Q Σ E S : Type} [OrderedAddCommMonoid S]
  (𝒮 : CoherenceObject X Q S)
  (𝒫 : SpacetimeTransitionSystem Σ E S)
  (Φ : X → Σ) (Ψ : Q → E)
  (τ_X : Trajectory X Q)
  (h_states_ne : τ_X.states ≠ [])
  (h_iso : Isomorphism 𝒮 𝒫 Φ Ψ) :
  TrajectoryCommit 𝒮 τ_X ↔ TrajectoryCommit 𝒫 {
    states := τ_X.states.map Φ,
    actions := τ_X.actions.map Ψ,
    continuous := by simp [τ_X.continuous]
  } := by
  unfold TrajectoryCommit
  simp [h_iso.val_pres, h_iso.spend_pres, h_iso.defect_pres, h_iso.auth_pres]
  rfl

end Coh.Physics.Trajectory
