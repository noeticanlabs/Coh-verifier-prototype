import Mathlib
import Coh.Boundary.CohAtom
import Coh.Boundary.CohSpinorDynamics

namespace Coh.Boundary

/--
## Phase Loom (The Trajectory Manifold)
\boxed{ \mathcal{L} = (\Phi, \mathcal{T}, \mathcal{I}, \mathcal{H}) }
-/

/--
### Loom Persistence Law
All stored atoms must remain verifiable.
-/
def loom_persistence {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) 
  (L : Set (CohAtom S)) : Prop :=
  ∀ A ∈ L, A.executable

/--
### Loom Phase Locality (Matching)
Retrieval is alignment-based, not lookup.
-/
def phase_match {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} {A : CohAtom S}
  (spinor : CohSpinor S A) (epsilon : ENNRat) (target_atom : CohAtom S) : Prop :=
  target_atom.final_state = spinor.state_hash ∧ 
  spinor.amplitude > epsilon

/--
### Theorem: Loom Persistence Preserved
Weaving an executable atom preserves the persistence property.
-/
theorem loom_persistence_preserved {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash)
  (L : Set (CohAtom S)) (A : CohAtom S)
  (h_L : loom_persistence S L) (h_A : A.executable) :
  loom_persistence S (L ∪ {A}) := by
  unfold loom_persistence
  intro A' h_in
  cases h_in with
  | inl h_old => exact h_L A' h_old
  | inr h_new => 
    rw [Set.mem_singleton_iff] at h_new
    rw [h_new]
    exact h_A

/--
### Theorem: Phase Alignment Consistency
Retrieval only returns atoms that are spatially consistent with the spinor's current state.
-/
theorem phase_alignment_consistent {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} {A : CohAtom S}
  (spinor : CohSpinor S A) (epsilon : ENNRat) (target_atom : CohAtom S)
  (h_match : phase_match spinor epsilon target_atom) :
  target_atom.final_state = spinor.state_hash := by
  exact h_match.left

end Coh.Boundary
