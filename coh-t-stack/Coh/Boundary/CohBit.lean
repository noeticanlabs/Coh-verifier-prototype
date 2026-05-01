import Mathlib
import Coh.Templates
import Coh.Boundary.RationalInf

namespace Coh.Boundary

/--
## Coh-Bit Governed Runtime System v1.0
\boxed{ \textbf{CohBit} = \text{A verifier-governed computational cell.} }

A Coh-bit does not merely encode information; it certifies whether a proposed state 
transition is admissible under a resource-bounded law. 

### The Transition Cycle:
Proposal → Admission → Verification → Commit → Receipt → Memory

Coh bits do not compute by permission of hardware alone; they compute by passing 
a formal verifier law.
-/

inductive RvStatus where
  | unknown : RvStatus
  | accept : RvStatus
  | reject : RvStatus
  deriving DecidableEq

/--
### Coh System v1.0
The rigorous environment for governed state transitions.
-/
structure CohSystem (X Action Cert Hash : Type) where
  V : X -> ENNRat -- Valuation (Remaining Coherence Reserve)
  spend : Action -> ENNRat
  defect : Action -> ENNRat
  delta_hat : Action -> ENNRat
  authority : Action -> ENNRat
  exec : X -> Action -> X -- Correctness law
  
  hash_state : X -> Hash
  hash_action : Action -> Hash
  
  rv_verify : Cert -> RvStatus
  certifies : Cert -> X -> Action -> X -> Prop
  
  -- Hard Invariants (C8-C10, C19)
  nonneg_spend : ∀ a, 0 ≤ spend a
  nonneg_defect : ∀ a, 0 ≤ defect a
  nonneg_delta : ∀ a, 0 ≤ delta_hat a
  nonneg_authority : ∀ a, 0 ≤ authority a
  defect_bound : ∀ a, defect a ≤ delta_hat a
  
  -- Identity Axioms
  id_action : Action
  id_exec : ∀ x, exec x id_action = x
  id_spend_zero : spend id_action = 0
  id_defect_zero : defect id_action = 0
  id_authority_zero : authority id_action = 0

/--
### CohBit v1.0 structure
Carries both data and proof obligations.
-/
structure CohBit {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) where
  from_state : X
  action : Action
  to_state : X
  cert : Cert
  
  -- Proof Obligations
  rv_ok : S.rv_verify cert = RvStatus.accept -- C7
  cert_ok : S.certifies cert from_state action to_state -- C6
  exec_ok : S.exec from_state action = to_state -- C17 (The Reactor Wall)
  defect_ok : S.defect action ≤ S.delta_hat action -- C8
  
  -- Budget Admissibility (C9)
  -- V_post + spend ≤ V_pre + defect + authority
  margin_ok :
    S.V to_state + S.spend action ≤
    S.V from_state + S.defect action + S.authority action

/--
### Theorem: Identity CohBit Exists
Grounds the transition graph in a formally admissible neutral atom. [PROVED]
-/
theorem identity_exists {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) (x : X) (cert : Cert)
  (h_rv : S.rv_verify cert = RvStatus.accept)
  (h_cert : S.certifies cert x S.id_action x) :
  CohBit S := {
    from_state := x,
    action := S.id_action,
    to_state := x,
    cert := cert,
    rv_ok := h_rv,
    cert_ok := h_cert,
    exec_ok := S.id_exec x,
    defect_ok := by rw [S.id_defect_zero]; exact S.nonneg_delta S.id_action,
    margin_ok := by
      rw [S.id_spend_zero, S.id_defect_zero, S.id_authority_zero]
      simp only [add_zero]
      exact le_refl (S.V x)
  }

/--
### Theorem: Trajectory Stability (The Crown Jewel)
A verified chain preserves the cumulative coherence budget.
Telescoping sum proof. [PROVED]
-/
theorem chain_stability {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash}
  (chain : List (CohBit S))
  (h_nonempty : chain ≠ [])
  (h_cont : ∀ i, (chain.get? i).map (·.to_state) = (chain.get? (i+1)).map (·.from_state))
  (h_finite : ∀ x, S.V x ≠ ⊤) :
  S.V (chain.getLast h_nonempty).to_state + (chain.map (fun b => S.spend b.action)).sum ≤
  S.V (chain.head h_nonempty).from_state + (chain.map (fun b => S.defect b.action)).sum + (chain.map (fun b => S.authority b.action)).sum := by
  match chain with
  | [b] =>
    simp only [List.map_singleton, List.sum_singleton, List.getLast_singleton, List.head_cons]
    exact b.margin_ok
  | b :: b' :: tail =>
    have htail_ne : b' :: tail ≠ [] := by simp
    have h_tail_cont : ∀ i, ((b' :: tail).get? i).map (·.to_state) =
                             ((b' :: tail).get? (i+1)).map (·.from_state) := by
      intro i; exact h_cont (i + 1)
    have ih := chain_stability (b' :: tail) htail_ne h_tail_cont h_finite
    have b_margin := b.margin_ok
    have step_link : b.to_state = b'.from_state := by
      have h := h_cont 0; simp at h; exact h
    rw [step_link] at b_margin
    have hlast : (b :: b' :: tail).getLast h_nonempty = (b' :: tail).getLast htail_ne :=
      List.getLast_cons_cons b b' tail
    -- Fully expand ih so its sums match the goal after simp
    simp only [List.map_cons, List.sum_cons, List.head_cons] at ih ⊢
    rw [hlast]
    -- ih  : V(getLast t).to + (sp_b' + Σsp_t) ≤ V(b'.from) + (df_b' + Σdf_t) + (au_b' + Σau_t)
    -- b_margin : V(b'.from) + sp_b ≤ V(b.from) + df_b + au_b  [after step_link rewrite]
    -- goal: V(getLast t).to + (sp_b + (sp_b' + Σsp_t)) ≤ V(b.from) + (df_b + (df_b' + Σdf_t)) + (au_b + (au_b' + Σau_t))
    -- coh_compose_linear composes ordered monoid inequalities
    have key := coh_compose_linear b_margin ih
    -- key is in the shape: V(getLast t) + (sp_b + (sp_b' + Σsp_t)) ≤ V(b.from) + ...
    -- The add_comm_group shape may differ; use add_assoc + add_comm to bridge
    calc S.V (List.getLast (b' :: tail) htail_ne).to_state +
         (S.spend b.action + (S.spend b'.action + List.sum (List.map (fun x => S.spend x.action) tail)))
        ≤ S.V b.from_state +
          (S.defect b.action + (S.defect b'.action + List.sum (List.map (fun x => S.defect x.action) tail))) +
          (S.authority b.action + (S.authority b'.action + List.sum (List.map (fun x => S.authority x.action) tail))) :=
          coh_compose_linear b_margin ih
  | [] => contradiction

end Coh.Boundary
