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
  | Burned
  deriving DecidableEq

/--
## Hardened Authority Grant
-/
structure AuthorityGrant (Hash : Type) where
  atom_id : Hash
  authority : ENNRat
  receipt_hash : Hash
  signer : Hash
  domain_id : Hash
  policy_hash : Hash
  expires_at : ℕ
  grant_hash : Hash

/--
## Decoherence Certificate v1.1 (Burned Resources)
-/
structure DecoherenceCertificate {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash}
  (e : CohEntanglement X Action Cert Hash S) where
  cause : DecoherenceCause
  post_local_margins : List ENNRat
  burned_shared_defect : ENNRat
  burned_shared_authority : ENNRat
  redistributed_shared_defect : ENNRat
  redistributed_shared_authority : ENNRat
  split_witness : Hash

  -- Law: Shared resources are nullified, not redistributed
  budget_nullification :
    burned_shared_defect = e.shared_defect ∧
    burned_shared_authority = e.shared_authority ∧
    redistributed_shared_defect = 0 ∧
    redistributed_shared_authority = 0

  all_valid : ∀ m ∈ post_local_margins, m ≥ 0

  /-- Certificate-atom linkage: post-split local margins correspond to atoms -/
  post_local_sum_ok :
    post_local_margins.sum =
      (e.atoms.map (fun a =>
        (S.V a.initial_state + a.cumulative_defect + a.cumulative_authority
          - S.V a.final_state - a.cumulative_spend))).sum

  /-- Accounting conservation: local margins plus burned shared equal the joint margin formula -/
  sum_conservation :
    post_local_margins.sum + burned_shared_defect + burned_shared_authority =
      calculate_joint_margin e

structure QuarantineReceipt {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash}
  (e : CohEntanglement X Action Cert Hash S) where
  cause : DecoherenceCause
  failed_participants : List Hash
  failed_margins : List ENNRat
  at_least_one_failed : failed_participants.length > 0

def hard_split_possible {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash}
  (e : CohEntanglement X Action Cert Hash S) : Prop :=
  ∀ a ∈ e.atoms, executable a

/--
## Theorem: Decoherence Accounting (Exact)
Local margins plus burned shared resources reconcile exactly to the joint margin value.
[PROVED] — by structural equalities carried in the certificate and joint law.
-/
theorem decoherence_accounting_exact {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash}
  (e : CohEntanglement X Action Cert Hash S)
  (cert : DecoherenceCertificate e)
  (h_joint : joint_margin_ok e) :
  cert.post_local_margins.sum + cert.burned_shared_defect + cert.burned_shared_authority = e.joint_margin := by
  obtain ⟨h_val, _h_nonneg⟩ := h_joint
  -- Replace calculate_joint_margin using joint law
  simpa [h_val] using cert.sum_conservation

end Coh.Boundary
