import Mathlib
import Coh.Boundary.CohBit

namespace Coh.Boundary

/--
## CohAtom v1.0 (The Stable Walk)
\boxed{ \textbf{CohAtom} = \text{The minimal closed executable trajectory of CohBits.} }
-/

/--
### CohAtom structure
Finite, canonical, cryptographically committed transition complex.
-/
structure CohAtom {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) where
  bits : List (CohBit S)
  
  -- A3: Non-empty unless Identity (Represented here as non-empty for simplicity)
  nonempty_bits : bits ≠ []
  
  -- Identity
  initial_state : X
  final_state : X
  
  -- A4, A5: Boundary Laws
  first_ok : bits.head (nonempty_bits) |>.from_state = initial_state
  last_ok : bits.getLast (nonempty_bits) |>.to_state = final_state
  
  -- A6: State Continuity Law
  continuous :
    ∀ (i : ℕ) (h : i + 1 < bits.length),
      (bits.get ⟨i, lt_trans (Nat.lt_succ_self i) h⟩).to_state =
      (bits.get ⟨i + 1, h⟩).from_state

/--
### Cumulative Metrics (A11-A14)
Auxiliary definitions for budget calculation.
-/
def sum_spend {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (bits : List (CohBit S)) : ENNRat :=
  bits.map (λ b => S.spend b.action) |>.sum

def sum_defect {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (bits : List (CohBit S)) : ENNRat :=
  bits.map (λ b => S.defect b.action) |>.sum

def sum_authority {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (bits : List (CohBit S)) : ENNRat :=
  bits.map (λ b => S.authority b.action) |>.sum

/--
### Cumulative Budget Law (A16-A17)
The telescoping sum of local CohBit margins.
V_n + sum spend ≤ V_0 + sum defect + sum authority
-/
def cumulative_margin_ok {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (a : CohAtom S) : Prop :=
  S.V a.final_state + sum_spend a.bits ≤
  S.V a.initial_state + sum_defect a.bits + sum_authority a.bits

/--
### Theorem: All Bits Admissible implies Atom Admissible
The core telescoping proof.
-/
theorem cohatom_all_bits_executable {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (a : CohAtom S) :
  cumulative_margin_ok a := by
  unfold cumulative_margin_ok
  -- Induction on bits list would show the telescoping property
  sorry

/--
### Theorem: Identity CohAtom is Valid
Grounds the trajectory category.
-/
theorem identity_cohatom_valid {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) (x : X) (b_id : CohBit S)
  (h_from : b_id.from_state = x)
  (h_to : b_id.to_state = x)
  (h_id_action : b_id.action = S.id_action) :
  let atom_id : CohAtom S := {
    bits := [b_id],
    nonempty_bits := by simp,
    initial_state := x,
    final_state := x,
    first_ok := by simp [h_from],
    last_ok := by simp [h_to],
    continuous := by intros i h; simp at h; contradiction -- Only one bit
  }
  cumulative_margin_ok atom_id := by
  unfold cumulative_margin_ok
  simp [sum_spend, sum_defect, sum_authority]
  rw [h_id_action, S.id_spend_zero, S.id_defect_zero, S.id_authority_zero]
  simp

end Coh.Boundary
