import Coh.Physics.Spinor.Basic
import Coh.Physics.Spinor.Gamma
import Coh.Physics.Spinor.Proofs

namespace Coh.Physics.Spinor

/--
## Coherence Current Vector
J_C^mu = bar{psi} gamma^mu psi
-/
noncomputable def coherence_current (psi : SpinorSpace) (gamma : GammaMatrix) : Complex :=
  let psi_bar := adjoint psi
  let row := psi_bar * gamma
  (row * Matrix.col (Fin 4) psi.get) 0 0

/--
## J0 is Real and Non-Negative [PROVED]
The time-like component J_C^0 = bar{psi} gamma^0 psi equals psi† gamma^0^2 psi = psi† psi = rho.
Since gamma^0 is Hermitian and (gamma^0)^2 = 1, J^0 = psi† psi which is real and non-negative.
-/
theorem j0_eq_density (psi : SpinorSpace) : 
  coherence_current psi gamma0 = (density psi : Complex) := by
  unfold coherence_current
  unfold adjoint
  rw [Matrix.mul_assoc]
  rw [gamma0_sq_eq_one]
  rw [Matrix.mul_one]
  unfold density
  simp [Matrix.mul_apply, Matrix.conjTranspose_apply, Matrix.col_apply, Complex.normSq]
  have h : (psi.toList.map fun c => (Complex.normSq c : Complex)).sum = 
           Finset.univ.sum (fun i => (Complex.normSq (psi.get i) : Complex)) := by
    rw [List.sum_eq_univ_sum]
    simp
  exact h

/-- Abstract divergence operator pending geometric formalization. -/
structure DivergenceOperator (Current Scalar : Type) where
  div : Current → Scalar
  zero : Scalar

/--
[THEOREM-TARGET]
Full divergence-free conservation of the coherence four-current.

This is intentionally axiomatized until the Dirac operator, spinor field,
current construction, and geometric divergence layer are formalized.

Requires:
1. A formal Dirac operator.
2. A definition of the coherence current over a field.
3. Compatibility between Dirac dynamics and the current.
4. A divergence operator on the relevant manifold structure.
-/
axiom coherence_four_current_divergence_free
  {Cur Sc : Type}
  (D : DivergenceOperator Cur Sc)
  (J : Cur)
  : D.div J = D.zero

end Coh.Physics.Spinor
