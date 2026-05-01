import Mathlib
import Coh.Boundary.CohBit

namespace Coh.Boundary

/--
## CohAtom v1.0
The minimal closed executable trajectory of CohBits.
-/
structure CohAtom {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) where
  bits : List (CohBit S)
  nonempty_bits : bits ≠ []
  initial_state : X
  final_state : X
  first_ok : (bits.head nonempty_bits).from_state = initial_state
  last_ok : (bits.getLast nonempty_bits).to_state = final_state
  continuous :
    ∀ (i : ℕ) (h : i + 1 < bits.length),
      (bits.get ⟨i, lt_trans (Nat.lt_succ_self i) h⟩).to_state =
      (bits.get ⟨i + 1, h⟩).from_state

/--
### Cumulative Metrics
-/
def sum_spend {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (bits : List (CohBit S)) : ENNRat :=
  (bits.map (fun b => S.spend b.action)).sum

def sum_defect {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (bits : List (CohBit S)) : ENNRat :=
  (bits.map (fun b => S.defect b.action)).sum

def sum_authority {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (bits : List (CohBit S)) : ENNRat :=
  (bits.map (fun b => S.authority b.action)).sum

/--
### Cumulative Budget Law
-/
def cumulative_margin_ok {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} (a : CohAtom S) : Prop :=
  S.V a.final_state + sum_spend a.bits ≤
  S.V a.initial_state + sum_defect a.bits + sum_authority a.bits

/--
### Theorem: Identity CohAtom is Valid
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
    continuous := by sorry
  }
  cumulative_margin_ok atom_id := by sorry

end Coh.Boundary
