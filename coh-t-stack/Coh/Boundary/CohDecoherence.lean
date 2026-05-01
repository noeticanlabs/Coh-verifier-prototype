import Mathlib
import Coh.Boundary.CohEntanglement

namespace Coh.Boundary

inductive DecoherenceCause where
  | CouplingWitnessExpired
  | SharedAuthorityExhausted
  | ManualSeverance
  | PolicyChanged

inductive DecoherenceState where
  | Coherent
  | SplitCertified
  | Quarantined
  deriving DecidableEq

structure AuthorityGrant {Hash : Type} where
  atom_id : Hash
  authority : ENNRat
  receipt_hash : Hash

structure DecoherenceCertificate {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) where
  cause : DecoherenceCause
  post_local_margins : List ENNRat
  released_shared_defect : ENNRat
  released_shared_authority : ENNRat
  split_witness : Hash
  budget_isolation : released_shared_defect = e.shared_defect ∧ released_shared_authority = e.shared_authority
  all_valid : ∀ m ∈ post_local_margins, m ≥ 0

structure QuarantineReceipt {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) where
  cause : DecoherenceCause
  failed_participants : List Hash
  failed_margins : List ENNRat
  at_least_one_failed : failed_participants.length > 0

def hard_split_possible {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) : Prop :=
  ∀ a ∈ e.atoms, cumulative_margin_ok (S := S) {
    bits := a.bits,
    nonempty_bits := a.nonempty_bits,
    initial_state := a.initial_state,
    final_state := a.final_state,
    first_ok := a.first_ok,
    last_ok := a.last_ok,
    continuous := a.continuous
  }

theorem hard_split_safety {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) :
  joint_margin_ok e → hard_split_possible e → True := by
  sorry

end Coh.Boundary
