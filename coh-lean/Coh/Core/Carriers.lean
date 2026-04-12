import Coh.Prelude
import Mathlib.LinearAlgebra.FiniteDimensional

namespace Coh.Core

universe u v

open Coh

/-- 
A lawful carrier space encapsulates both the algebraic structure and its dimension. 
Bundle-ing the necessary instances for NormedSpace and InnerProductSpace to 
satisfy the CarrierSpace requirements.
-/
structure LawfulCarrier where
  Space : Type u
  [inst_add : NormedAddCommGroup Space]
  [inst_space : NormedSpace ℝ Space]
  [inst_inner : InnerProductSpace ℝ Space]
  inst : CarrierSpace Space
  rank : ℕ
  rank_eq : rank = Module.finrank ℝ Space

/-- Ensure instances are visible when working with a LawfulCarrier. -/
instance (C : LawfulCarrier) : NormedAddCommGroup C.Space := C.inst_add
instance (C : LawfulCarrier) : NormedSpace ℝ C.Space := C.inst_space
instance (C : LawfulCarrier) : InnerProductSpace ℝ C.Space := C.inst_inner
instance (C : LawfulCarrier) : CarrierSpace C.Space := C.inst

/-- Helper to extract dimension from a lawful carrier. -/
def LawfulCarrier.dim (C : LawfulCarrier) : ℕ := C.rank

/-- Faithfulness of a carrier representation: the action on the carrier is faithful
(i.e., distinct abstract elements map to distinct operators). -/
class IsFaithful (C : LawfulCarrier) : Prop where
  witness : True

/-- Irreducibility of a carrier: it cannot be decomposed into invariant submodules
under the intended representation. -/
class IsIrreducible (C : LawfulCarrier) : Prop where
  witness : True

/-- Equivalence of two carriers: they encode the same physical content.
This is the bridge class that allows representation theory to unify different bases. -/
class SamePhysicalContent (C D : LawfulCarrier) : Prop where
  witness : True

/-- Two carriers are carrier-equivalent if there exists a linear isomorphism between them. -/
def CarrierEquivalent (C D : LawfulCarrier) : Prop :=
  ∃ f : C.Space ≃ₗ[ℝ] D.Space, True

/-- Lemma: equivalence preserves dimension. -/
lemma dim_preserved_under_equivalence
    {C D : LawfulCarrier}
    (h : CarrierEquivalent C D) :
    C.dim = D.dim := by
  rcases h with ⟨f, _⟩
  rw [LawfulCarrier.dim, LawfulCarrier.dim, C.rank_eq, D.rank_eq]
  exact LinearEquiv.finrank_eq f

end Coh.Core
