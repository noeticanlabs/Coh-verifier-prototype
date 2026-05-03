import Mathlib
import Coh.Templates
import Coh.Boundary.CohBit

namespace Coh.Boundary

/--
## Atom Kind
-/
inductive AtomKind where
  | ExecutableTrajectory
  | SummaryTrajectory
  | Identity
  deriving DecidableEq

/--
## CohAtom v1.1
The minimal closed executable trajectory of CohBits.
-/
structure CohAtom {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) where
  bits : List (CohBit S)
  nonempty_bits : bits ≠ []
  initial_state : X
  final_state : X
  cumulative_spend : ENNRat
  cumulative_defect : ENNRat
  cumulative_delta_hat : ENNRat
  cumulative_authority : ENNRat
  margin_total : ENNRat
  kind : AtomKind
  compression_certificate : Option Hash

  -- Boundary Continuity Invariants
  first_ok : (bits.head nonempty_bits).from_state = initial_state
  last_ok : (bits.getLast nonempty_bits).to_state = final_state
  continuous :
    ∀ (i : ℕ) (h : i + 1 < bits.length),
      (bits.get ⟨i, lt_trans (Nat.lt_succ_self i) h⟩).to_state =
      (bits.get ⟨i + 1, h⟩).from_state

/--
## Metric Recomputation (Law)
-/
def recompute_metrics {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (a : @CohAtom X Action Cert Hash S) : ENNRat × ENNRat × ENNRat × ENNRat :=
  let spend := (a.bits.map (fun b => S.spend b.action)).sum
  let defect := (a.bits.map (fun b => S.defect b.action)).sum
  let delta_hat := (a.bits.map (fun b => S.delta_hat b.action)).sum
  let authority := (a.bits.map (fun b => S.authority b.action)).sum
  (spend, defect, delta_hat, authority)

def metrics_ok {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (a : @CohAtom X Action Cert Hash S) : Prop :=
  let (s, d, dh, auth) := recompute_metrics a
  s = a.cumulative_spend ∧ 
  d = a.cumulative_defect ∧ 
  dh = a.cumulative_delta_hat ∧ 
  auth = a.cumulative_authority ∧
  d ≤ dh

/--
## Budget Admissibility Law
-/
def budget_valid {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (a : @CohAtom X Action Cert Hash S) : Prop :=
  let v_pre := (a.bits.head a.nonempty_bits).v_pre
  let v_post := (a.bits.getLast a.nonempty_bits).v_post
  a.margin_total = v_pre + a.cumulative_defect + a.cumulative_authority - v_post - a.cumulative_spend ∧
  a.margin_total ≥ 0

/--
## Execution Gates
-/
def retrieval_valid {X Action Cert Hash : Type _} {S : CohSystem X Action Cert Hash} 
  (a : @CohAtom X Action Cert Hash S) : Prop :=
  match a.kind with
  | AtomKind.ExecutableTrajectory => metrics_ok a
  | AtomKind.Identity => metrics_ok a
  | AtomKind.SummaryTrajectory => a.compression_certificate.isSome ∧ metrics_ok a

def mutation_valid {X Action Cert Hash : Type _} {S : CohSystem X Action Cert Hash} 
  (a : @CohAtom X Action Cert Hash S) : Prop :=
  match a.kind with
  | AtomKind.ExecutableTrajectory => 
    retrieval_valid a ∧ 
    (∀ b ∈ a.bits, S.rv_verify b.cert = RvStatus.accept) ∧
    budget_valid a
  | AtomKind.Identity =>
    retrieval_valid a ∧ 
    (∀ b ∈ a.bits, S.rv_verify b.cert = RvStatus.accept) ∧
    budget_valid a
  | AtomKind.SummaryTrajectory => False

def executable {X Action Cert Hash : Type _} {S : CohSystem X Action Cert Hash} 
  (a : @CohAtom X Action Cert Hash S) : Prop :=
  mutation_valid a

/--
### Theorem: Atom Metrics Stability
Bridging the Bit-level stability to the Atom-level cumulative fields. [PROVED]
-/
theorem atom_metrics_stability {X Action Cert Hash : Type _} {S : CohSystem X Action Cert Hash}
  (a : @CohAtom X Action Cert Hash S)
  (h_mut : mutation_valid a) 
  (h_finite : ∀ x, S.V x ≠ ⊤) :
  (a.bits.getLast a.nonempty_bits).v_post + a.cumulative_spend ≤ 
  (a.bits.head a.nonempty_bits).v_pre + a.cumulative_defect + a.cumulative_authority := by
  -- 1. Unpack mutation_valid
  cases h_kind : a.kind
  case ExecutableTrajectory =>
    unfold mutation_valid retrieval_valid at h_mut
    rw [h_kind] at h_mut
    simp at h_mut
    obtain ⟨h_ret, _, _⟩ := h_mut
    obtain ⟨h_spend, h_defect, _, h_auth, _⟩ := h_ret
    rw [← h_spend, ← h_defect, ← h_auth]
    
    -- Apply chain_stability
    exact chain_stability a.bits a.nonempty_bits a.continuous h_finite
  case Identity =>
    unfold mutation_valid retrieval_valid at h_mut
    rw [h_kind] at h_mut
    simp at h_mut
    obtain ⟨h_ret, _, _⟩ := h_mut
    obtain ⟨h_spend, h_defect, _, h_auth, _⟩ := h_ret
    rw [← h_spend, ← h_defect, ← h_auth]
    
    exact chain_stability a.bits a.nonempty_bits a.continuous h_finite
  case SummaryTrajectory =>
    unfold mutation_valid at h_mut
    rw [h_kind] at h_mut
    simp at h_mut
    contradiction

end Coh.Boundary
