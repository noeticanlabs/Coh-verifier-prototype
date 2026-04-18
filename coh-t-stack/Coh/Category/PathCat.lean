import Coh.Category.CohCat

/-!
# Coh.Category.PathCat

Raw dynamics category: all possible (x, r, x') steps without RV verification.

This is the ambient category of traces before filtering by legal transitions.
- CohDyn(A) is the verified subcategory (only steps where RV = true)
- PathCat(A) is the full subcategory of all traces

Design goals:
- No verification: any triple (x, r, x') is a valid RawStep
- Trace composition is simple list append (like DynHom)
- The inclusion functor i_A : CohDyn(A) ⟶ PathCat(A) forgets the RV proof

This module provides the raw dynamics layer for the chain-of-constructions pipeline.
-/

namespace Coh.Category

/- Base object carrier (same as CohDyn) -/
abbrev PathObj := CohObj

/- A raw step without RV verification -/
structure RawStep (A : PathObj) (x y : A.X) where
  r : A.R
  -- No verification field: any receipt is allowed

namespace RawStep
/- Convenience accessors -/
def src {A : PathObj} {x y : A.X} (s : RawStep A x y) : A.X := x
def dst {A : PathObj} {x y : A.X} (s : RawStep A x y) : A.X := y
end RawStep

/- Raw paths (traces) as inductive lists of raw steps -/
inductive PathHom (A : PathObj) : A.X → A.X → Type (max u v)
  | nil (x : A.X) : PathHom A x x
  | cons {x y z : A.X} (s : RawStep A x y) (p : PathHom A y z) : PathHom A x z

namespace PathHom

variable {A : PathObj}

/- Append composition: recurse on the left for definitional laws -/
def comp {x y z : A.X} : PathHom A y z → PathHom A x y → PathHom A x z
  | PathHom.nil _,      p => p
  | PathHom.cons s q,   p => PathHom.cons s (comp q p)

@[simp] lemma comp_nil {x y : A.X} (p : PathHom A x y) :
  comp (PathHom.nil y) p = p := rfl

lemma comp_id_right : ∀ {x y : A.X} (p : PathHom A x y),
  comp p (PathHom.nil x) = p
  | _, _, PathHom.nil _ => rfl
  | _, _, PathHom.cons s q => by
      simp [comp, comp_id_right q]

lemma assoc : ∀ {w x y z : A.X}
  (h : PathHom A y z) (g : PathHom A x y) (f : PathHom A w x),
  comp h (comp g f) = comp (comp h g) f
  | _, _, _, _, PathHom.nil _, g, f => rfl
  | _, _, _, _, PathHom.cons s h', g, f => by
      simp [comp, assoc h' g f]

end PathHom

/- SmallCategory instance over A.X using raw paths as homs -/
def PathCat (A : PathObj) : SmallCategory A.X :=
  { Hom := fun x y => PathHom A x y
  , id := fun x => PathHom.nil x
  , comp := fun g f => PathHom.comp g f
  , id_comp := by
      intro x y f
      cases f with
      | _nil x' => simp [PathHom.comp]
      | @cons x' y' z' s p =>
          simp [PathHom.comp]
  , comp_id := by
      intro x y f
      cases f with
      | _nil x' => simp [PathHom.comp]
      | @cons x' y' z' s p =>
          simpa [PathHom.comp, PathHom.comp_id_right p]
  , assoc := by
      intro w x y z f g h
      cases f with
      | _nil _ => simp [PathHom.comp]
      | @cons _ _ _ s f' =>
          simp [PathHom.comp, PathHom.assoc f' g h] }

/- Inclusion functor: embed verified dynamics into raw dynamics -/
namespace InclFunctor

open Coh.Category

/- Map a verified step to a raw step by forgetting RV proof -/
def mapStep {A : PathObj} {x y : A.X} (s : Step A x y) : RawStep A x y :=
  { r := s.r }

/- Map a verified path to a raw path -/
def mapPath {A : PathObj} :
  ∀ {x y : A.X}, DynHom A x y → PathHom A x y
  | x, _, DynHom.nil _ => PathHom.nil _
  | x, _, DynHom.cons s p => PathHom.cons (mapStep s) (mapPath p)

/- Inclusion functor from CohDyn(A) to PathCat(A) -/
def toFunctor (A : PathObj) : SmallFunctor (CohDyn A) (PathCat A) :=
{ obj := id
, map := by
    intro x y h
    exact mapPath h
, map_id := by
    intro x
    rfl
, map_comp := by
    intro x y z g h
    induction g with
    | _nil _ => simp [CohDyn, DynHom.comp, PathHom.comp, mapPath]
    | @cons x' y' z' s p ih =>
        simp [CohDyn, DynHom.comp, PathHom.comp, mapPath, ih] }

end InclFunctor

end Coh.Category
