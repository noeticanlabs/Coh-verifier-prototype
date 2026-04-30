import Mathlib
import Coh.Boundary.CohSpinor

namespace Coh.Boundary

/--
## CohSpinor Dynamics (The Internal Evolution)
\boxed{ \psi_{t+1} = \mathcal{U}(b_t, x_t, \psi_t) }
-/

/--
### Spin Evolution Property
A transition only rotates the spinor if the bit is admissible.
-/
def spin_evolves {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} {A : CohAtom S}
  (psi psi' : CohSpinor S A) (bit : CohBit S) : Prop :=
  if S.rv_verify bit.cert = RvStatus.accept then
    -- Norm preservation bound (S11)
    psi'.amplitude ≤ psi.amplitude + S.delta_hat bit.action
  else
    psi' = psi

/--
### Theorem: Spin Norm Bounded
The fundamental law of orientation stability.
-/
theorem spin_norm_bounded {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} {A : CohAtom S}
  (psi psi' : CohSpinor S A) (bit : CohBit S)
  (h_evolve : spin_evolves psi psi' bit) :
  psi'.amplitude ≤ psi.amplitude + S.delta_hat bit.action := by
  unfold spin_evolves at h_evolve
  split at h_evolve
  · exact h_evolve
  · rw [h_evolve]
    apply le_add_of_nonneg_right
    exact S.nonneg_delta bit.action

/--
### Theorem: Invalid Bit No Spin Evolution
Discrete verification overrides continuous dynamics.
-/
theorem invalid_bit_no_spin_evolution {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} {A : CohAtom S}
  (psi psi' : CohSpinor S A) (bit : CohBit S)
  (h_rv : S.rv_verify bit.cert ≠ RvStatus.accept)
  (h_evolve : spin_evolves psi psi' bit) :
  psi' = psi := by
  unfold spin_evolves at h_evolve
  split at h_evolve
  · contradiction
  · exact h_evolve

end Coh.Boundary
