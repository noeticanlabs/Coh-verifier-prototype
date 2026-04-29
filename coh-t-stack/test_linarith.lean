import Mathlib.Tactic.Linarith
import Coh.Boundary.LawOfGenesis
open Coh.Boundary
theorem test_linarith {{G P R : Type}} [OrderedAddCommMonoid R] (obj : GenesisObject G P R) (g1 g2 g3 : G) (p1 p2 : P) (h1 : GenesisAdmissible obj g1 p1 g2) (h2 : GenesisAdmissible obj g2 p2 g3) : obj.M g3 + (obj.C p1 + obj.C p2) ≤ obj.M g1 + (obj.D p1 + obj.D p2) := by unfold GenesisAdmissible at h1 h2; obtain ⟨_, h1_ineq⟩ := h1; obtain ⟨_, h2_ineq⟩ := h2; linarith