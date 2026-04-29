import Mathlib.Algebra.Order.Monoid.Defs

namespace Coh.Boundary

/--
The Law of Genesis formalizes forward admissible generation.
A transition (g, p, g') is admissible if it satisfies the hard compatibility relation
and the resource inequality: M(g') + C(p) ≤ M(g) + D(p).
-/
structure GenesisObject (G P R : Type) [OrderedAddCommMonoid R] where
  Gamma : G → P → G → Prop
  M : G → R
  C : P → R
  D : P → R

def GenesisAdmissible {G P R : Type} [OrderedAddCommMonoid R] 
  (obj : GenesisObject G P R) (g : G) (p : P) (g' : G) : Prop :=
  obj.Gamma g p g' ∧ obj.M g' + obj.C p ≤ obj.M g + obj.D p

/--
The composition of two Genesis-admissible transitions is Genesis-admissible
if cost and slack are additive.
-/
theorem genesis_composition {G P R : Type} [OrderedAddCommMonoid R] 
  (obj : GenesisObject G P R) (g1 g2 g3 : G) (p1 p2 : P)
  (h1 : GenesisAdmissible obj g1 p1 g2)
  (h2 : GenesisAdmissible obj g2 p2 g3) :
  obj.M g3 + (obj.C p1 + obj.C p2) ≤ obj.M g1 + (obj.D p1 + obj.D p2) := by
  unfold GenesisAdmissible at h1 h2
  obtain ⟨_, h1_ineq⟩ := h1
  obtain ⟨_, h2_ineq⟩ := h2
  rw [add_comm (obj.C p1), ← add_assoc]
  refine le_trans (add_le_add_right h2_ineq _) ?_
  rw [add_assoc, add_comm (obj.D p2), ← add_assoc, ← add_assoc]
  exact add_le_add_right h1_ineq _

end Coh.Boundary
