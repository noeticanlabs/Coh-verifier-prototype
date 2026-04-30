import Mathlib

namespace Coh.Boundary

abbrev SpaceIndex := Fin 4
def MatterField : Type := SpaceIndex → ℚ
def Tensor2 : Type := SpaceIndex → SpaceIndex → ℚ

structure EffMetric (g : Tensor2) : Prop where
  symmetric : ∀ mu nu, g mu nu = g nu mu

def stressEnergyTensor (Psi : MatterField) : Tensor2 :=
  fun mu nu => Psi mu * Psi nu

def curvatureTerm (g : Tensor2) : Tensor2 :=
  fun mu nu => g mu nu

structure FieldEquation (g : Tensor2) (Psi : MatterField) (kappa l : ℚ) : Prop where
  holds : ∀ mu nu, g mu nu = kappa * (stressEnergyTensor Psi mu nu) + l * (curvatureTerm g mu nu)

/--
## Stress-Energy Tensor Symmetry [PROVED]
The stress-energy tensor T_mu_nu = Psi_mu * Psi_nu is symmetric.
Proof: mul_comm on ℚ.
-/
theorem stressEnergyTensor_symmetric (Psi : MatterField) :
  ∀ mu nu, stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu := by
  intros mu nu
  unfold stressEnergyTensor
  apply mul_comm

/--
## Effective Metric Symmetry [PROVED]
The field equation g_mu_nu = k*T_mu_nu + l*g_mu_nu preserves symmetry.
Proof: because T is symmetric and the equation holds point-wise for both
orderings, g_mu_nu = g_nu_mu follows by algebraic equality.
-/
theorem field_equation_effective_metric {g : Tensor2} {Psi : MatterField} {kappa l : ℚ}
  (h : FieldEquation g Psi kappa l)
  (hl : l < 1) :
  EffMetric g := by
  constructor
  intro mu nu
  have hmunu := h.holds mu nu
  have hnumu := h.holds nu mu
  have hT : stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu :=
    stressEnergyTensor_symmetric Psi mu nu
  unfold curvatureTerm at hmunu hnumu
  rw [hT] at hmunu
  have h_factor : (1 - l) * (g mu nu - g nu mu) = 0 := by linarith
  have h_ne : 1 - l ≠ 0 := by linarith
  have h_zero : g mu nu - g nu mu = 0 := (mul_eq_zero.mp h_factor).resolve_left h_ne
  exact sub_eq_zero.mp h_zero

/--
## Field Equation Uniqueness [PROVED]
If l < 1, the field equation g = k*T + l*g has a unique solution.
Proof: both g1 and g2 satisfy the same point-wise linear equation.
Subtracting: g1_mn - g2_mn = l*(g1_mn - g2_mn), so (1-l)*(g1-g2) = 0.
Since l < 1, 1-l ≠ 0, so g1 = g2.
-/
theorem field_equation_unique {g1 g2 : Tensor2} {Psi : MatterField} {kappa l : ℚ}
  (h1 : FieldEquation g1 Psi kappa l)
  (h2 : FieldEquation g2 Psi kappa l)
  (hl : l < 1) :
  g1 = g2 := by
  funext mu nu
  have e1 := h1.holds mu nu
  have e2 := h2.holds mu nu
  unfold curvatureTerm at e1 e2
  have h_factor : (1 - l) * (g1 mu nu - g2 mu nu) = 0 := by linarith
  have h_ne : 1 - l ≠ 0 := by linarith
  have h_zero : g1 mu nu - g2 mu nu = 0 := (mul_eq_zero.mp h_factor).resolve_left h_ne
  exact sub_eq_zero.mp h_zero

end Coh.Boundary
