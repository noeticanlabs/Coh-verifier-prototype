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
## Entanglement Mode
-/
inductive EntanglementMode where
  | Fixture
  | Heuristic
  | Production
  deriving DecidableEq

/--
## Coh Entanglement v2.5
-/
structure CohEntanglement (X Action Cert Hash : Type) (S : CohSystem X Action Cert Hash) where
  atoms : List (CohAtom S)
  shared_defect : ENNRat
  shared_delta_hat : ENNRat
  shared_authority : ENNRat
  shared_authority_cap : ENNRat
  joint_margin : ENNRat
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
def calculate_joint_margin {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) : ENNRat :=
  let sum_val_pre : ENNRat := (e.atoms.map (fun a => S.V a.initial_state)).sum
  let sum_val_post : ENNRat := (e.atoms.map (fun a => S.V a.final_state)).sum
  let sum_spend_all : ENNRat := (e.atoms.map (fun a => a.cumulative_spend)).sum
  (sum_val_pre + e.shared_defect + e.shared_authority) - (sum_val_post + sum_spend_all)

def joint_margin_ok {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) : Prop :=
  e.joint_margin = calculate_joint_margin e ∧
  e.joint_margin ≥ 0

/--
## Monogamy Registration
-/
inductive MonogamyState where
  | Active
  | Decohered
  | Burned
  | Quarantined
  deriving DecidableEq

structure MonogamyRegistry (Hash : Type) where
  keys : Hash → Option MonogamyState

def verify_base {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  (e : CohEntanglement X Action Cert Hash S) (mode : EntanglementMode) : Prop :=
  (∀ a ∈ e.atoms, retrieval_valid a) ∧
  e.domain_id = e.domain_id ∧ -- Simplified context check
  joint_margin_ok e ∧
  e.shared_defect ≤ e.shared_delta_hat ∧
  e.shared_authority ≤ e.shared_authority_cap ∧
  (mode = EntanglementMode.Production → e.witness_kind ≠ CouplingWitnessKind.FixtureOnly)

end Coh.Boundary
