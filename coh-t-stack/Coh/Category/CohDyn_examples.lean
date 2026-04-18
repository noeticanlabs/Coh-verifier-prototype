import Coh.Category.CohDyn

/-!
# Coh.Category.CohDyn_examples

A tiny example demonstrating a minimal governed system and the
verification of category laws.

The example system:
- X = {a, b, c} (3 states)
- R = {r1, r2} (2 receipts)
- RV enforces a linear chain: a → b → c

The purpose is to show that the machinery works on a concrete
finite instance and to illustrate the category laws by computation.
-/

namespace Coh.Category

import Mathlib.Data.Real.NNReal

-- A minimal example: 3 states, 2 receipts
def tinyX : Type := Fin 3

-- Receipts: r0 and r1
def tinyR : Type := Bool  -- true = r1, false = r0

-- Potential: simple NNReal numbers (0, 1, 2)
def tinyV (x : tinyX) : NNReal := x.val

-- RV: only forward steps a→b and b→c are legal
def tinyRV (x : tinyX) (r : tinyR) (y : tinyX) : Bool :=
  match x, r, y with
  | 0, false, 1 => true   -- a --r0--> b
  | 1, false, 2 => true   -- b --r0--> c
  | 0, true, 1  => true   -- a --r1--> b
  | 1, true, 2  => true   -- b --r1--> c
  | _, _, _     => false

-- Construct the tiny CohObj
def tinyCohObj : CohObj :=
  { X := tinyX
  , R := tinyR
  , V := tinyV
  , RV := tinyRV }

-- Compute cost of a single step: max 0 (V dst - V src)
-- Step 0→1: V(1) - V(0) = 1 - 0 = 1
-- Step 1→2: V(2) - V(1) = 2 - 1 = 1
def tinyStep1Cost : NNReal := step_cost tinyCohObj tinyV tinyStep1

def tinyStep2Cost : NNReal := step_cost tinyCohObj tinyV tinyStep2

-- Cost of the full path (0→1→2) = 1 + 1 = 2
def tinyPathCost : NNReal := path_cost tinyCohObj tinyV tinyPath

-- Verify subadditivity: cost(f ∘ g) ≤ cost(f) + cost(g)
-- For our path, composition is the path itself, so equality holds
theorem tiny_path_cost_subadditive :
  path_cost tinyCohObj tinyV (DynHom.comp tinyPath tinyPath) ≤
    path_cost tinyCohObj tinyV tinyPath + path_cost tinyCohObj tinyV tinyPath := by
  -- Since the path is non-nil, the left side is exactly 2 * tinyPathCost
  -- and the right side is tinyPathCost + tinyPathCost = 2 * tinyPathCost
  simp [path_cost, DynHom.comp, tinyPathCost]
  -- Both sides compute to 2 + 2 = 4; equality holds
  rfl

-- Verify the construction works
#check tinyCohObj

-- Verify that CohDyn builds successfully
#check CohDyn tinyCohObj

-- Verify we can form a step
def tinyStep1 : Step tinyCohObj 0 1 :=
  { r := false, ok := rfl }  -- r0 from a to b

def tinyStep2 : Step tinyCohObj 1 2 :=
  { r := false, ok := rfl }  -- r0 from b to c

-- Build a path (a → b → c)
def tinyPath : DynHom tinyCohObj 0 2 :=
  DynHom.cons tinyStep1 (DynHom.cons tinyStep2 (DynHom.nil 2))

-- Verify identity paths work
def tinyIdPath (x : tinyX) : DynHom tinyCohObj x x :=
  DynHom.nil x

-- Verify that id_comp works on this concrete instance
theorem tiny_id_comp : DynHom.comp (DynHom.nil 1) tinyStep1 = tinyStep1 := rfl

-- Verify that comp_id works on this concrete instance
theorem tiny_comp_id : DynHom.comp tinyStep1 (DynHom.nil 0) = tinyStep1 := rfl

-- Verify that associativity holds on a concrete chain
theorem tiny_assoc :
  let p1 := DynHom.nil 2;
  let p2 := tinyStep2;
  let p3 := tinyStep1;
  DynHom.comp p1 (DynHom.comp p2 p3) = DynHom.comp (DynHom.comp p1 p2) p3 := rfl

-- Verify that we can map along identity homomorphism
def tinyHomId : CohHom tinyCohObj tinyCohObj :=
  { fX := fun x => x
  , fR := fun r => r
  , preserves := by
      intro x y r h
      -- rv is preserved because it's exactly the same RV
      simpa using h }

-- Check that mapStep works
#check DynFunctor.mapStep tinyHomId tinyStep1

-- Check that mapDyn works
#check DynFunctor.mapDyn tinyHomId tinyPath

-- The identity homomorphism should map paths identically
theorem tiny_map_id_path :
  DynFunctor.mapDyn tinyHomId tinyPath = tinyPath := rfl

end Coh.Category
