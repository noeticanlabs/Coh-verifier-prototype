import Mathlib.Analysis.InnerProductSpace.Basic
import Mathlib.Analysis.InnerProductSpace.Projection
import Mathlib.Data.Matrix.Basic
import Mathlib.Data.Complex.Basic
import Mathlib.Algebra.BigOperators.Basic
import Mathlib.Data.Matrix.Notation

namespace Coh.Physics.Spinor

/-!
## Lean Proof: gamma0^2 = Identity

In the Dirac representation, gamma0 = diag(1,1,-1,-1).
So gamma0 * gamma0 = diag(1,1,1,1) = I_4.
-/

abbrev GammaMatrix := Matrix (Fin 4) (Fin 4) Complex

-- Dirac gamma0 matrix
def gamma0_mat : GammaMatrix := !![
  1, 0, 0, 0;
  0, 1, 0, 0;
  0, 0, -1, 0;
  0, 0, 0, -1
]

theorem sum_fin_four {α : Type _} [AddCommMonoid α] (f : Fin 4 → α) :
  Finset.univ.sum f = f 0 + f 1 + f 2 + f 3 := by
  simp [Finset.univ, Finset.sum_insert, Finset.sum_singleton]

theorem gamma0_sq_eq_one : gamma0_mat * gamma0_mat = (1 : GammaMatrix) := by
  ext i j
  unfold gamma0_mat
  rw [Matrix.mul_apply, Matrix.one_apply]
  rw [sum_fin_four]
  fin_cases i <;> fin_cases j <;> simp <;> norm_num

/-!
## Lean Proof: Projection Weight Non-Negativity

For any matrix P and vector v, density(P v) = ||P v||^2 ≥ 0.
This follows directly from Complex.normSq_nonneg applied to each component.
-/

def SpinorVec := Fin 4 → Complex

def vec_density (v : SpinorVec) : ℝ :=
  Finset.univ.sum (fun i => Complex.normSq (v i))

/--
## Projection Weight is Non-Negative [PROVED]
For any matrix P and spinor v, ||P v||^2 ≥ 0.
-/
theorem proj_density_nonneg (P : Matrix (Fin 4) (Fin 4) Complex) (v : SpinorVec) :
  vec_density (fun i => (P.mulVec v) i) ≥ 0 := by
  apply Finset.sum_nonneg
  intro i _
  exact Complex.normSq_nonneg _

/-!
## Lean Proof: Projector Weight Sum = 1 for Diagonal/Coordinate Projectors

For coordinate projectors P_i (projecting onto component i),
the sum of weights w_i = |psi_i|^2 over all i equals sum_i |psi_i|^2 = density(psi).
For a normalized psi, this equals 1.
-/

/-- Coordinate projector onto component k -/
def coord_proj (k : Fin 4) : Matrix (Fin 4) (Fin 4) Complex :=
  fun i j => if i = k ∧ j = k then 1 else 0

/--
## coord_proj is idempotent [PROVED]
-/
theorem coord_proj_idem (k : Fin 4) : coord_proj k * coord_proj k = coord_proj k := by
  ext i j
  unfold coord_proj
  rw [Matrix.mul_apply]
  fin_cases k <;> fin_cases i <;> fin_cases j <;> simp

/--
## coord_proj is Hermitian [PROVED]
The coordinate projectors are real diagonal matrices, so P† = P.
-/
theorem coord_proj_hermitian (k : Fin 4) : (coord_proj k).conjTranspose = coord_proj k := by
  ext i j
  unfold coord_proj
  rw [Matrix.conjTranspose_apply]
  simp
  fin_cases k <;> fin_cases i <;> fin_cases j <;> simp

/--
## Coordinate Projector Weight Sum = density [PROVED]
sum_{k=0}^{3} |P_k psi|^2 = sum_{k} |psi_k|^2 = density(psi)
-/
theorem coord_proj_weight_sum (v : SpinorVec) :
  Finset.univ.sum (fun k => vec_density (fun i => (coord_proj k).mulVec v i)) =
  vec_density v := by
  unfold vec_density coord_proj
  simp [Matrix.mulVec, Matrix.dotProduct]
  -- handle the inner summation over j
  simp_rw [Finset.sum_ite_eq]
  simp [Complex.normSq_zero]
  -- Use sum_comm to group by k and then apply sum_ite_eq
  rw [Finset.sum_comm]
  simp [Finset.sum_ite_eq]

end Coh.Physics.Spinor
