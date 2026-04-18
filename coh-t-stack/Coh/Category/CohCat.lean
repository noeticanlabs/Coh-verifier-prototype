import Coh.Kernel.T1_Category
import Mathlib.Data.Real.NNReal
import Coh.Category.CohDyn

/-!
# Coh.Category: Base category of governed systems (objects with V and RV)

Objects carry:
- a state type X
- a receipt type R
- a potential V : X → NNReal (nonnegative real-valued potential for enrichment)
- a verifier RV : X → R → X → Bool (discrete acceptance predicate)

Morphisms f : A ⟶ B are pairs (fX, fR) that preserve acceptance:
  RV_A x r x' = true → RV_B (fX x) (fR r) (fX x') = true

This file builds a SmallCategory structure (in the style of T1) over these
objects and homomorphisms, so users can reason categorically about translators
between governed systems.
-/

namespace Coh.Category

universe u v

/- Base objects: (X, R, V, RV) -/
structure CohObj where
  X  : Type u
  R  : Type v
  V  : X → NNReal
  RV : X → R → X → Bool

/- Verification-preserving morphisms between base objects -/
structure CohHom (A B : CohObj) where
  fX : A.X → B.X
  fR : A.R → B.R
  preserves : ∀ {x x' : A.X} {r : A.R},
    A.RV x r x' = true →
    B.RV (fX x) (fR r) (fX x') = true

namespace CohHom

/- Identity morphism -/
def id (A : CohObj) : CohHom A A :=
  { fX := id, fR := id, preserves := by intro x x' r h; simpa using h }

/- Composition of morphisms -/
def comp {A B C : CohObj} (g : CohHom B C) (f : CohHom A B) : CohHom A C :=
  { fX := fun x => g.fX (f.fX x)
  , fR := fun r => g.fR (f.fR r)
  , preserves := by
      intro x x' r h
      have hB : B.RV (f.fX x) (f.fR r) (f.fX x') = true := f.preserves h
      exact g.preserves hB }

end CohHom

/- SmallCategory instance following Coh.Kernel.SmallCategory style -/
open Coh.Kernel

def CohCat : SmallCategory CohObj :=
  { Hom := fun A B => CohHom A B
  , id := fun A => CohHom.id A
  , comp := fun g f => CohHom.comp g f
  , id_comp := by
      intro A B f; cases f; rfl
  , comp_id := by
      intro A B f; cases f; rfl
  , assoc := by
      intro A B C D f g h; cases f; cases g; cases h; rfl }

/-!
## Functorial Dynamics

The functor `Dyn : CohCat → Cat` maps each governed system `A` to its dynamics
category `CohDyn(A)`, and each homomorphism `f : CohHom A B` to the induced functor
on dynamics via `DynFunctor.toSmallFunctor`.
-/

namespace Dyn

/- Lift a CohHom to a functor between dynamics categories -/
def lift {A B : CohObj} (f : CohHom A B) : SmallFunctor (CohDyn A) (CohDyn B) :=
  DynFunctor.toSmallFunctor f

/- Functoriality: identity maps to identity -/
theorem lift_id (A : CohObj) : lift (CohHom.id A) = {
    obj := id
    map := fun x y h => h
    map_id := by intro x; rfl
    map_comp := by intro x y z g h; rfl
  } := by
  apply SmallFunctor.ext <;> rfl

/- Functoriality: composition preserves -/
theorem lift_comp {A B C : CohObj} (f : CohHom A B) (g : CohHom B C) :
  lift (CohHom.comp f g) = {
    obj := (lift g).obj ∘ (lift f).obj
    map := fun x y h => (lift g).map ((lift f).map h)
    map_id := by intro x; simp [lift, DynFunctor.toSmallFunctor]
    map_comp := by intro x y z g' h'; simp [lift, DynFunctor.toSmallFunctor]
  } := by
  apply SmallFunctor.ext <;> rfl

end Dyn

end Coh.Category
