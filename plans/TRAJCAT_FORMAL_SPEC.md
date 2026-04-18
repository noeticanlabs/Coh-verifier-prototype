# Trajectory Category (TrajCat) — Formal Specification in Lean

## Overview

Formal definition of the trajectory search engine as a category in Lean 4, building on existing `CohDyn` (verified dynamics) and `PathCat` (raw paths).

## Core Definitions

### States and Actions

```lean
/- State: carrier object in the system -/
structure State (A : CohObj) where
  value : A.X
  budget : A.R  -- remaining potential/budget

/- Action: possible transition from a state -/
structure Action (A : CohObj) where
  label : String
  transition : A.R → A.X → A.X  -- state transformer
  
/- Candidate: enumerated action from current state -/
structure Candidate (A : CohObj) where
  action : Action A
  next_state : State A
  receipt : A.R  -- micro-receipt from action
```

### Trajectory Step

```lean
/- Single step in trajectory with verification witness -/
structure TrajStep (A : CohObj) where
  state : State A
  action : Action A
  receipt : A.R
  
  -- Constraint witness: C1-C6 pass/fail status
  witnesses : C₁ → C₆ → WitnessStatus
  where WitnessStatus := Pass | Fail | Unknown
```

### Trajectory

```lean
/- Trajectory: verified sequence of steps -/
structure Trajectory (A : CohObj) where
  id : String
  steps : List (TrajStep A)
  depth : Nat
  
  -- Computed
  is_admissible : Bool  -- all steps pass C₁-C₅
  first_failure : Option Nat
  score : Float
```

### TrajCat (Category of Trajectories)

```lean
/- Objects: states -/
abbrev TrajObj (A : CohObj) := State A

/- Morphisms: admissible trajectories -/
structure TrajHom (A : TrajObj) (x y : TrajObj) where
  trajectory : Trajectory A
  src : x
  dst : y
  is_valid : trajectory.is_admissible = true
```

## Admissibility Constraints

### C₁-C₅ Mapped to Lean

| Lean Constraint | Formal |
|---------------|--------|
| C₁ Schema | `receipt.is_valid_schema` |
| C₂ Sigs | `receipt.signatures.all Authenticated` |
| C₃ Profile | `receipt.canon_profile_hash = expected` |
| C₄ State | `state.budget' ≤ state.budget + receipt.authority` |
| C₅ Digest | `chain_digest.link_valid` |

### Verified Transition Relation

```lean
/- Valid edge: all constraints pass -/
def valid_edge {A : CohObj} (s : State A) (a : Action A) : State A → Prop :=
  fun s' =>
    ∃ (r : A.R) (w : WitnessMap),
      w.C₁ = Pass ∧ w.C₂ = Pass ∧ w.C₃ = Pass ∧ 
      w.C₄ = Pass ∧ w.C₅ = Pass ∧
      s'.budget ≤ s.budget + r.authority
```

## Search Operations

### Expand (Generate Candidates)

```lean
/- Enumerate candidate actions from state -/
def expand {A : CohObj} (s : State A) (K : Nat) : List (Candidate A) :=
  enumerate_actions s |>.take K
```

### Verify Edge

```lean
/- Verify single edge against constraints -/
def verify_edge {A : CohObj} (c : Candidate A) : TrajStep A :=
  let r := c.receipt;
  let w := extract_witnesses r;
  TrajStep.mk c.action c.next_state r w
```

### Beam Search

```lean
/- Bounded beam search -/
def beam_search 
  (A : CohObj) 
  (s₀ : State A) 
  (depth : Nat) 
  (beam : Nat) 
  : List (Trajectory A) :=
  
  go depth [Trajectory.nil s₀]
  where
  go : Nat → List (Trajectory A) → List (Trajectory A)
    | 0, ts => ts
    | n+1, ts =>
      let ts' := ts.flatMap (fun t =>
        let s := t.last_state;
        expand s beam |>.map (fun c =>
          verify_edge c
        )
      )
      -- Filter admissible, sort by score, keep beam
      ts'.filter is_admissible 
          |>.sort_by scoreDesc 
          |>.take beam
      go n ts'
```

## Scoring Function

```lean
/- Score trajectory -/
def score {A : TrajObj} 
  (t : Trajectory A) 
  (w_goal : Float := 1.0)
  (w_risk : Float := 0.5)
  (w_cost : Float := 0.2)
  : Float :=
  
  let goal_score := t.depth.cast Float
  let risk_penalty := t.first_failure.map (·.cast Float) |>.getOrElse 0
  let cost_penalty := t.depth.cast Float * w_cost
  
  w_goal * goal_score - w_risk * risk_penalty - cost_penalty
```

## Category Laws

### Identity

```lean
def TrajHom.id {A : TrajObj} (x : TrajObj) : TrajHom A x x :=
  TrajHom.mk (Trajectory.nil x) x x rfl
```

### Composition

```lean
def TrajHom.comp {A : TrajObj} {x y z : TrajObj}
  (f : TrajHom A x y) (g : TrajHom A y z) : TrajHom A x z :=
  -- Compose trajectories, preserving admissibility
  TrajHom.mk (f.trajectory ++ g.trajectory) x z
    (by simp [is_admissible, *])
```

## Integration Points

- Uses `CohObj` from `CohCat`
- Extends `CohDyn` (verified dynamics) with search
- Trajectories compose as morphisms

## Notes for Implementation

- `enumerate_actions` is domain-specific (implemented per use case)
- Beam width controls search width
- Depth bounds search horizon
- Score weights tunable per application