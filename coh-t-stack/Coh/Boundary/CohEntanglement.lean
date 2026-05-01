import Mathlib
import Coh.Boundary.CohAtom

namespace Coh.Boundary

/--
## Coupling Witness Kind
-/
inductive CouplingWitnessKind where
  | FixtureOnly
  | HeuristicCorrelation
  | CertifiedNonSeparability
  deriving DecidableEq

/--
## Structural CohAtom
-/
structure StructuralCohAtom (X Action Cert Hash : Type) (S : CohSystem X Action Cert Hash) where
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
## Coh Entanglement v2.4
-/
structure CohEntanglement (X Action Cert Hash : Type) (S : CohSystem X Action Cert Hash) where
  atoms : List (StructuralCohAtom X Action Cert Hash S)
  shared_defect : ENNRat
  shared_delta_hat : ENNRat
  shared_authority : ENNRat
  shared_authority_cap : ENNRat
  domain_id : Hash
  policy_hash : Hash
  monogamy_scope : Hash
  witness_kind : CouplingWitnessKind
  coupling_witness : Hash

  -- Security Invariants
  shared_defect_bounded : shared_defect ≤ shared_delta_hat
  shared_authority_bounded : shared_authority ≤ shared_authority_cap

/--
## Joint Admissibility Law (E4)
-/
def joint_margin_ok {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) : Prop :=
  let sum_val_pre : ENNRat := (e.atoms.map (fun a => S.V a.initial_state)).sum
  let sum_val_post : ENNRat := (e.atoms.map (fun a => S.V a.final_state)).sum
  let sum_spend_all : ENNRat := (e.atoms.map (fun a => sum_spend a.bits)).sum
  
  sum_val_post + sum_spend_all ≤ 
  sum_val_pre + e.shared_defect + e.shared_authority

/--
## Lemma: Individual Inadmissibility
-/
lemma entangled_non_separability {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) :
  joint_margin_ok e → ¬ (∀ a ∈ e.atoms, cumulative_margin_ok (S := S) {
    bits := a.bits,
    nonempty_bits := a.nonempty_bits,
    initial_state := a.initial_state,
    final_state := a.final_state,
    first_ok := a.first_ok,
    last_ok := a.last_ok,
    continuous := a.continuous
  }) → True := by
  sorry

end Coh.Boundary
