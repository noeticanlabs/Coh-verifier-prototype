import Mathlib.Analysis.Complex.Basic
import Mathlib.Data.Vector
import Mathlib.Analysis.InnerProductSpace.Basic
import Mathlib.Data.Matrix.Basic
import Coh.Physics.Spinor.Proofs

namespace Coh.Physics.Spinor

def SpinorSpace := Vector Complex 4
def Projector := Matrix (Fin 4) (Fin 4) Complex

def is_projector (P : Projector) : Prop :=
  P * P = P ∧ P.conjTranspose = P

/--
## Spinor Density
rho_C = sum |psi_i|^2 (detector-frame pre-measure).
-/
def density (psi : SpinorSpace) : ℝ :=
  (psi.toList.map (fun c => Complex.normSq c)).sum

/--
## Coh Spinor State (normalized)
-/
structure CohSpinor where
  state : SpinorSpace
  normalized : (state.toList.map (fun c => Complex.normSq c)).sum = 1

/--
## Positive Density Theorem [PROVED]
rho_C >= 0 for any spinor state.
Proof: sum of Complex.normSq values, each non-negative.
-/
theorem positive_density_theorem (psi : SpinorSpace) : density psi ≥ 0 := by
  unfold density
  apply List.sum_nonneg
  intro c _
  simp [Complex.normSq_nonneg]

/--
## Normalized CohSpinor has density 1 [PROVED]
By definition of the CohSpinor structure.
-/
theorem cohspinor_density_eq_one (psi : CohSpinor) : density psi.state = 1 :=
  psi.normalized

/--
## Projection Weight
w_i = ||P_i psi||^2 = density(P_i psi)
-/
noncomputable def projection_weight (P : Projector) (psi : SpinorSpace) : ℝ :=
  let projected := P * Matrix.col (Fin 4) psi.get
  density (Vector.ofFn (fun i => projected i 0))

/--
## Projection Weight is Non-Negative [PROVED]
Born weights are non-negative — they are sums of normSq values.
-/
theorem projection_weight_nonneg (P : Projector) (psi : SpinorSpace) :
  projection_weight P psi ≥ 0 := by
  unfold projection_weight
  apply positive_density_theorem

/--
## Projector Resolution
A complete, mutually orthogonal family of orthogonal projectors.
-/
structure ProjectorResolution where
  Ps : List Projector
  each_projector : ∀ P ∈ Ps, is_projector P
  pairwise_orthogonal : ∀ P ∈ Ps, ∀ Q ∈ Ps, P ≠ Q → P * Q = 0
  sums_to_identity : Ps.sum = (1 : Projector)
end Coh.Physics.Spinor
