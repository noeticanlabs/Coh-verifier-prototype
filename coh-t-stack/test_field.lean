import Mathlib

abbrev ENNRat := WithTop NNRat

abbrev SpaceIndex := Fin 4
def MatterField : Type := SpaceIndex → ENNRat
def Tensor2 : Type := SpaceIndex → SpaceIndex → ENNRat

structure EffMetric (g : Tensor2) : Prop where
  symmetric : ∀ mu nu, g mu nu = g nu mu

def stressEnergyTensor (Psi : MatterField) : Tensor2 :=
  fun mu nu => Psi mu * Psi nu

def curvatureTerm (g : Tensor2) : Tensor2 :=
  fun mu nu => g mu nu

structure FieldEquation (g : Tensor2) (Psi : MatterField) (kappa l : ENNRat) : Prop where
  holds : ∀ mu nu, g mu nu = kappa * (stressEnergyTensor Psi mu nu) + l * (curvatureTerm g mu nu)

theorem stressEnergyTensor_symmetric (Psi : MatterField) :
  ∀ mu nu, stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu := by
  intros mu nu
  unfold stressEnergyTensor
  apply mul_comm

theorem field_equation_effective_metric {g : Tensor2} {Psi : MatterField} {kappa l : ENNRat}
  (hl : l < 1) (h : FieldEquation g Psi kappa l) :
  EffMetric g := by
  constructor
  intro mu nu
  have eq1 : g mu nu = kappa * stressEnergyTensor Psi mu nu + l * g mu nu := h1
  have eq2 : g nu mu = kappa * stressEnergyTensor Psi nu mu + l * g nu mu := h2
  have T_symm : stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu :=
    stressEnergyTensor_symmetric Psi mu nu
  -- From symmetry: T_μν = T_νμ
  -- Rearranging both equations: g(μν) - l*g(μν) = κ*T_μν and g(νμ) - l*g(νμ) = κ*T_νμ
  -- Since l < 1, we can solve: g_μν = κ*T_μν / (1-l) = κ*T_νμ/(1-l) = g_νμ
  have eq1' : g mu nu * (1 - l) = kappa * stressEnergyTensor Psi mu nu := by
    calc g mu nu
      _ = kappa * stressEnergyTensor Psi mu nu + l * g mu nu := eq1.symm
      _ = kappa * stressEnergyTensor Psi mu nu + l * g mu nu - l * g mu nu := by ring
      _ = kappa * stressEnergyTensor Psi mu nu := by ring
  have eq2' : g nu mu * (1 - l) = kappa * stressEnergyTensor Psi nu mu := by
    calc g nu mu
      _ = kappa * stressEnergyTensor Psi nu mu + l * g nu mu := eq2.symm
      _ = kappa * stressEnergyTensor Psi nu mu + l * g nu mu - l * g nu mu := by ring
      _ = kappa * stressEnergyTensor Psi nu mu := by ring
  rw [T_symm] at eq2'
  -- Now both sides equal κ*T_μν*(1-l), so the tensors are equal
  apply (mul_right_inj (by linarith)).mpr at eq1'
  apply (mul_right_inj (by linarith)).mpr at eq2'
  -- (1-l) > 0 since l < 1, so we can divide
  calc g mu nu
    _ = kappa * stressEnergyTensor Psi mu nu * (1 - l)⁻¹ := eq1'
    _ = kappa * stressEnergyTensor Psi nu mu * (1 - l)⁻¹ := by rw [T_symm]
    _ = g nu mu := eq2'.symm

theorem field_equation_unique {g1 g2 : Tensor2} {Psi : MatterField} {kappa l : ENNRat}
  (h1 : FieldEquation g1 Psi kappa l)
  (h2 : FieldEquation g2 Psi kappa l)
  (hl : l < 1) :
  g1 = g2 := by
  funext mu nu
  have e1 := h1.holds mu nu
  have e2 := h2.holds mu nu
  unfold curvatureTerm at e1 e2
  -- Both equations: g1_μν = κ*T_μν + l*g1_μν and g2_μν = κ*T_μν + l*g2_μν
  -- Rearranging: (1-l)*g1_μν = κ*T_μν = (1-l)*g2_μν
  -- Since l < 1, (1-l) > 0, so g1_μν = g2_μν
  have eq1' : g1 mu nu * (1 - l) = kappa * stressEnergyTensor Psi mu nu := by
    calc g1 mu nu
      _ = kappa * stressEnergyTensor Psi mu nu + l * g1 mu nu := e1.symm
      _ = kappa * stressEnergyTensor Psi mu nu + l * g1 mu nu - l * g1 mu nu := by ring
      _ = kappa * stressEnergyTensor Psi mu nu := by ring
  have eq2' : g2 mu nu * (1 - l) = kappa * stressEnergyTensor Psi mu nu := by
    calc g2 mu nu
      _ = kappa * stressEnergyTensor Psi mu nu + l * g2 mu nu := e2.symm
      _ = kappa * stressEnergyTensor Psi mu nu + l * g2 mu nu - l * g2 mu nu := by ring
      _ = kappa * stressEnergyTensor Psi mu nu := by ring
  -- Both equal κ*T_μν, so they are equal
  calc g1 mu nu
    _ = g2 mu nu * (1 - l)⁻¹ * (1 - l) := (mul_left_inj (by linarith)).mpr (by simp [eq1', eq2'])
    _ = g2 mu nu := by ring
