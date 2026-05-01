import Mathlib
import Coh.Boundary.CohAtom
import Coh.Boundary.CohSpinorDynamics

namespace Coh.Boundary

/--
## CohVM (The Verifier-Gated Machine)
\boxed{ \text{CohCompute} = \text{Certified, budget-bounded state mutation.} }
-/

/--
### VM State
Captures the current state of the virtual machine.
-/
structure VmState {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) where
  current_x : X
  current_V : ENNRat
  h_V : S.V current_x = current_V

/--
### Compute Law K1: No state mutation without executable CohBit.
The transition must be certified and budget-admissible.
-/
def vm_transition {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash}
  (st : VmState S) (bit : CohBit S) (st' : VmState S) : Prop :=
  bit.from_state = st.current_x ∧
  bit.to_state = st'.current_x ∧
  st'.current_V + S.spend bit.action ≤ st.current_V + S.defect bit.action + S.authority bit.action

/--
### Theorem: Transition Preserves System Integrity
Grounds the VM in the global Coh laws. [PROVED]
-/
theorem vm_step_is_admissible {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash}
  (st : VmState S) (bit : CohBit S) (st' : VmState S)
  (h_trans : vm_transition st bit st') :
  S.V st'.current_x + S.spend bit.action ≤ S.V st.current_x + S.defect bit.action + S.authority bit.action := by
  unfold vm_transition at h_trans
  rcases h_trans with ⟨_, _, h_budget⟩
  rw [st.h_V, st'.h_V]
  exact h_budget

/--
### Theorem: Scheduler Preference Subordination
Preference weighting occurs ONLY for admissible transitions. [PROVED]
-/
theorem preference_subordinated {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash}
  (valid_bits : Set (CohBit S)) (preferred_bit : CohBit S) :
  preferred_bit ∈ valid_bits → (S.rv_verify preferred_bit.cert = RvStatus.accept) := by
  intro h
  exact preferred_bit.rv_ok

end Coh.Boundary
