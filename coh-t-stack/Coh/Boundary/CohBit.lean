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
  
  -- Persistence and Safety Constants
  gamma_ps : ENNRat -- Persistence Sensitivity (Decay rate)
  pi_min : ENNRat   -- Admissibility Floor
  
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
### CohBit v1.3 structure
Hardened with security anchors and trace continuity fields.
-/
structure CohBit {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) where
  from_state : X
  action : Action
  to_state : X
  cert : Cert
  
  -- Security Anchors (NotebookLM v1.3)
  v_pre : ENNRat          -- Literal Pre-State Valuation
  v_post : ENNRat         -- Literal Post-State Valuation
  state_root : Hash       -- Cryptographic Pre-State Anchor
  v_proof : Cert          -- State-Read Proof (π_read)
  seq_index : ℕ           -- Trace Sequence Index
  
  -- Proof Obligations
  rv_ok : S.rv_verify cert = RvStatus.accept -- C7
  cert_ok : S.certifies cert from_state action to_state -- C6
  exec_ok : S.exec from_state action = to_state -- C17 (The Reactor Wall)
  defect_ok : S.defect action ≤ S.delta_hat action -- C8
  
  -- Valuation Integrity (NotebookLM v1.3)
  v_pre_ok : S.V from_state = v_pre
  v_post_ok : S.V to_state = v_post
  floor_ok : v_post ≥ S.pi_min
  
  -- Budget Admissibility (C9)
  -- V_post + spend ≤ V_pre + defect + authority
  margin_ok :
    v_post + S.spend action ≤
    v_pre + S.defect action + S.authority action

/--
### Theorem: Identity CohBit Exists
Grounds the transition graph in a formally admissible neutral atom. [PROVED]
-/
theorem identity_exists {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) (x : X) (cert : Cert)
  (h_rv : S.rv_verify cert = RvStatus.accept)
  (h_cert : S.certifies cert x S.id_action x)
  (h_floor : S.V x ≥ S.pi_min)
  (h_root : S.hash_state x = S.hash_state x) -- Trivial placeholder for hash stability
  (v_proof : Cert)
  (h_v_proof : S.V x = S.V x) : -- Placeholder for read-proof obligation
  CohBit S := {
    from_state := x,
    action := S.id_action,
    to_state := x,
    cert := cert,
    v_pre := S.V x,
    v_post := S.V x,
    state_root := S.hash_state x,
    v_proof := v_proof,
    seq_index := 0,
    rv_ok := h_rv,
    cert_ok := h_cert,
    exec_ok := S.id_exec x,
    defect_ok := by rw [S.id_defect_zero]; exact S.nonneg_delta S.id_action,
    v_pre_ok := rfl,
    v_post_ok := rfl,
    floor_ok := h_floor,
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
  (h_cont : ∀ (i : ℕ) (h : i + 1 < chain.length),
    (chain.get ⟨i, Nat.lt_of_succ_lt h⟩).to_state = (chain.get ⟨i + 1, h⟩).from_state)
  (h_finite : ∀ x, S.V x ≠ ⊤) :
  (chain.getLast h_nonempty).v_post + (chain.map (fun b => S.spend b.action)).sum ≤
  (chain.head h_nonempty).v_pre + (chain.map (fun b => S.defect b.action)).sum + (chain.map (fun b => S.authority b.action)).sum := by
  match chain with
  | [b] =>
    simp only [List.map_singleton, List.sum_singleton, List.getLast_singleton, List.head_cons]
    exact b.margin_ok
  | b :: b' :: tail =>
    have htail_ne : b' :: tail ≠ [] := by simp
    have h_tail_cont : ∀ (i : ℕ) (h : i + 1 < (b' :: tail).length),
      ((b' :: tail).get ⟨i, Nat.lt_of_succ_lt h⟩).to_state = ((b' :: tail).get ⟨i + 1, h⟩).from_state := by
      intro i h
      exact h_cont (i + 1) (Nat.succ_lt_succ h)
    have ih := chain_stability (b' :: tail) htail_ne h_tail_cont h_finite
    have b_margin := b.margin_ok
    
    -- Link v_post of b to v_pre of b'
    have v_link : b.v_post = b'.v_pre := by
      have step := h_cont 0 (by simp; exact Nat.succ_pos 0)
      simp [List.get] at step
      rw [← b.v_post_ok, ← b'.v_pre_ok, step]
    
    rw [v_link] at b_margin
    rw [List.getLast_cons_cons]
    simp only [List.map_cons, List.sum_cons, List.head_cons, List.getLast_cons] at ih ⊢
    exact Coh.coh_compose_linear b_margin ih

end Coh.Boundary
