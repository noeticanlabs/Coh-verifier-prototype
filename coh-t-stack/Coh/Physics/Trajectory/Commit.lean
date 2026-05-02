import Mathlib
import Coh.Boundary.LawOfCoherence
import Coh.Physics.Spacetime.SpacetimeTransition
import Coh.Physics.Isomorphism.Isomorphism

namespace Coh.Physics.Trajectory

open Classical
open Coh.Boundary
open Coh.Physics.Spacetime
open Coh.Physics.Isomorphism

/--
## CohBit Commit
-/
def CohCommit {X Q S : Type} [OrderedAddCommMonoid S]
  (V : X → S) (Spend Defect Authority : Q → S) (x : X) (q : Q) (y : X) : Prop :=
  V y + Spend q ≤ V x + Defect q + Authority q

/--
## Trajectory Structure
-/
structure Trajectory (X Q : Type) where
  states : List X
  actions : List Q
  continuous : states.length = actions.length + 1

/--
## Trajectory Commit (Telescoping Sum)
-/
def TrajectoryCommit {X Q S : Type} [OrderedAddCommMonoid S]
  (V : X → S) (Spend Defect Authority : Q → S) (ss : List X) (as : List Q) : Prop :=
  match ss with
  | [] => True
  | x :: xs =>
    V ((x :: xs).getLast (by simp)) + (as.map Spend).sum ≤ 
    V x + (as.map Defect).sum + (as.map Authority).sum

/--
## Theorem: Local Commit telescopes to Trajectory Commit
-/
theorem trajectory_commit_telescopes {X Q S : Type} [OrderedAddCommMonoid S]
  (V : X → S) (Spend Defect Authority : Q → S)
  (ss : List X) (as : List Q)
  (h_cont : ss.length = as.length + 1)
  (h_step : ∀ i : Fin as.length,
    CohCommit V Spend Defect Authority (ss.get ⟨i.1, by rw [h_cont]; exact Nat.lt_succ_of_lt i.2⟩) 
              (as.get i) 
              (ss.get ⟨i.1 + 1, by rw [h_cont]; exact Nat.succ_lt_succ i.2⟩)) :
  TrajectoryCommit V Spend Defect Authority ss as := by
  unfold TrajectoryCommit
  match h_s : ss with
  | [] => simp [h_s] at h_cont
  | x₀ :: xs =>
    simp [h_s]
    induction as generalizing x₀ xs
    case nil =>
      simp at h_cont
      match xs with
      | [] => simp; exact le_refl (V x₀)
      | _ :: _ => simp at h_cont
    case cons q qs ih =>
      match h_xs : xs with
      | x₁ :: xss =>
        simp at h_cont
        have h_head : CohCommit V Spend Defect Authority x₀ q x₁ := by
          have h_step_0 := h_step ⟨0, by simp⟩
          simp [h_s, h_xs] at h_step_0
          exact h_step_0
        have h_cont_tail : (x₁ :: xss).length = qs.length + 1 := by simp [h_cont]
        have ih_res := ih x₁ xss h_cont_tail (by
          intro i
          have h_step_i := h_step ⟨i.1 + 1, by simp; exact i.2⟩
          simp [h_s, h_xs] at h_step_i
          exact h_step_i)
        simp only [List.map_cons, List.sum_cons, List.head_cons, List.getLast_cons] at ih_res ⊢
        exact Coh.coh_compose_linear h_head ih_res
      | [] => simp at h_cont

/--
## Theorem: Isomorphism Preserves Trajectory Commit
-/
theorem isomorphism_preserves_trajectory_commit {X Q Sigma E S : Type} [OrderedAddCommMonoid S]
  (sysS : CoherenceObject X Q S)
  (sysP : SpacetimeTransitionSystem Sigma E S)
  (phi : X → Sigma) (psi : Q → E)
  (τ_X : Trajectory X Q)
  (h_iso : Isomorphism sysS sysP phi psi) :
  TrajectoryCommit sysS.V sysS.Spend sysS.Defect sysS.Authority τ_X.states τ_X.actions ↔ 
  TrajectoryCommit sysP.ℰ sysP.𝒜 sysP.δ sysP.𝒲 (τ_X.states.map phi) (τ_X.actions.map psi) := by
  unfold TrajectoryCommit
  match h : τ_X.states with
  | [] => rfl
  | x :: xs =>
    simp [h]
    simp [h_iso.val_pres, h_iso.spend_pres, h_iso.defect_pres, h_iso.auth_pres]
    simp [List.getLast_map]
    rfl

end Coh.Physics.Trajectory
