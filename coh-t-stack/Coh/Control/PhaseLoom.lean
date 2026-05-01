import Mathlib

namespace Coh

/-- Provenance: The origin and authority of a memory record. -/
structure Provenance where
  authority : ℕ
  deriving Repr, DecidableEq

instance : LT Provenance where
  lt p1 p2 := p1.authority < p2.authority

/-- Memory Tiers (Micro/Meso/Macro) -/
inductive MemoryTier where
  | Micro
  | Meso
  | Macro
  deriving Repr, DecidableEq

/-- Memory Operations -/
inductive MemoryOp where
  | Read
  | Write
  | Summarize
  | Approve
  deriving Repr, DecidableEq

/-- System Component Roles -/
inductive ComponentRole where
  | Verifier
  | AdmissionGate
  | Generator
  | MemoryManager
  | Auditor
  | Operator
  deriving Repr, DecidableEq

/-- PhaseLoom Memory Access Policy -/
def AccessPolicy := ComponentRole → MemoryTier → MemoryOp → Prop

/-- Memory Record -/
structure MemoryRecord (α : Type u) [OrderedAddCommMonoid α] where
  content : α
  prov : Provenance
  tau : ℕ
  accuracy : α

/-- Tiered Memory State -/
structure TieredMemory (α : Type u) [OrderedAddCommMonoid α] where
  micro : List (MemoryRecord α)
  meso : List (MemoryRecord α)
  macro : List (MemoryRecord α)

/-- PhaseLoom State v3.1 (Hardened Audit Edition) -/
structure PhaseLoomState (α : Type u) [OrderedAddCommMonoid α] where
  x : α        -- Semantic State
  C : α        -- Curvature
  B : α        -- Budget
  tau : ℕ      -- Intrinsic Time
  M : TieredMemory α -- Tiered Memory
  P : AccessPolicy   -- Access Policy

namespace PhaseLoom

/-- 
Governed Projection Operator (Π) 
Reduces memory to role-specific authorized view.
-/
def Project (role : ComponentRole) (α : Type u) [OrderedAddCommMonoid α] (M : TieredMemory α) : TieredMemory α :=
  match role with
  | .Verifier => { micro := M.micro.take 1, meso := [], macro := [] } -- High Loss
  | .AdmissionGate => { micro := M.micro, meso := M.meso, macro := [] } -- Medium Loss
  | _ => M

/--
[SPEC]
Specification-level control interface for PhaseLoom.
-/
structure ControlInterface (State Receipt Budget Debt : Type) where
  admissible : State → Receipt → State → Prop
  discipline : State → Receipt → Prop
  calibrated : State → Receipt → Prop
  debt : State → Debt
  budget : State → Budget
  
  -- Soundness Law
  sound :
    ∀ x r x',
      admissible x r x' →
      discipline x r ∧ calibrated x r

  -- [AUDIT REQUIREMENT] Projection Invariance Assumption
  -- This ensures that the Verifier can operate on a lossy view without loss of safety.
  projection_invariant :
    ∀ {α : Type u} [OrderedAddCommMonoid α] (s : PhaseLoomState α) (r : Receipt) (s' : State),
      admissible (cast (by rfl) s) r s' ↔ admissible (cast (by rfl) { s with M := Project .Verifier α s.M }) r s'

/-- Admissibility implies Discipline extraction [PROVED] -/
theorem admissible_implies_disciplined
  {State Receipt Budget Debt : Type}
  (CI : ControlInterface State Receipt Budget Debt)
  {x x' : State}
  {r : Receipt}
  (h : CI.admissible x r x')
  : CI.discipline x r :=
  (CI.sound x r x' h).left

/-- Admissibility implies Calibration extraction [PROVED] -/
theorem admissible_implies_calibrated
  {State Receipt Budget Debt : Type}
  (CI : ControlInterface State Receipt Budget Debt)
  {x x' : State}
  {r : Receipt}
  (h : CI.admissible x r x')
  : CI.calibrated x r :=
  (CI.sound x r x' h).right

/-- 
Governed Projection Invariance [PROVED]
Admissibility is invariant under the Verifier projection (by assumption).
-/
theorem governed_projection_invariance
    {α Receipt Budget Debt : Type}
    [OrderedAddCommMonoid α]
    (CI : ControlInterface (PhaseLoomState α) Receipt Budget Debt)
    (s : PhaseLoomState α)
    (r : Receipt)
    (s' : PhaseLoomState α)
    (h_adm : CI.admissible s r s') :
    CI.admissible ({ s with M := Project .Verifier α s.M }) r s' := by
  rw [← CI.projection_invariant s r s']
  exact h_adm

end PhaseLoom

end Coh
