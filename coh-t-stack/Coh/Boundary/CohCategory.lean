import Coh.Boundary.CohBit
import Coh.Boundary.CohAtom

namespace Coh.Boundary

/--
## CohCategory v1.3
The formal 2-category of certified transitions.

- **Objects (0-cells)**: Observable states `X`.
- **Morphisms (1-cells)**: Validated CohAtoms (Trajectories).
- **2-morphisms (2-cells)**: Trace Homotopies (Equivalence Proofs).
-/

structure CohTrace {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) (x y : X) where
  atom : CohAtom S
  start_match : atom.initial_state = x
  end_match : atom.final_state = y
  executable : executable atom

def id_trace {X Action Cert Hash : Type} (S : CohSystem X Action Cert Hash) (x : X) 
  (h_floor : S.V x ≥ S.pi_min) (cert v_proof : Cert)
  (h_rv : S.rv_verify cert = RvStatus.accept)
  (h_cert : S.certifies cert x S.id_action x) : 
  CohTrace S x x :=
  let bit := identity_exists S x cert h_rv h_cert h_floor rfl v_proof rfl
  let atom : CohAtom S := {
    bits := [bit],
    nonempty_bits := by simp,
    initial_state := x,
    final_state := x,
    cumulative_spend := 0,
    cumulative_defect := 0,
    cumulative_delta_hat := 0,
    cumulative_authority := 0,
    margin_total := 0,
    kind := AtomKind.Identity,
    compression_certificate := None,
    first_ok := rfl,
    last_ok := rfl,
    continuous := by intro i h; simp at h
  }
  {
    atom := atom,
    start_match := rfl,
    end_match := rfl,
    executable := by
      unfold executable mutation_valid retrieval_valid metrics_ok recompute_metrics
      simp
      rw [S.id_spend_zero, S.id_defect_zero, S.id_authority_zero]
      simp
      unfold budget_valid
      simp
      rw [S.id_spend_zero, S.id_defect_zero, S.id_authority_zero]
      simp
      exact bit.rv_ok
  }

/--
### Oplax Composition (V2 Projections)
Composition in Coh is not always strict. In Partially Observed systems, 
composing two traces may introduce a 'Supremum Envelope Defect' (D).
-/
def compose_oplax {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  {x y z : X} (t1 : CohTrace S x y) (t2 : CohTrace S y z) (D : ENNRat) : 
  Prop :=
  -- Strict composition is the case where D = 0
  -- Oplax composition allows the final budget to be slightly less than strict sum
  S.V z + (t1.atom.cumulative_spend + t2.atom.cumulative_spend) ≤ 
  S.V x + (t1.atom.cumulative_defect + t2.atom.cumulative_defect) + 
  (t1.atom.cumulative_authority + t2.atom.cumulative_authority) + D

/--
### Trace Homotopy (2-cells)
An equivalence between two traces connecting the same states.
Must preserve the net admissibility change.
-/
structure TraceHomotopy {X Action Cert Hash : Type} {S : CohSystem X Action Cert Hash} 
  {x y : X} (τ1 τ2 : CohTrace S x y) where
  equivalence : τ1.atom.initial_state = τ2.atom.initial_state ∧ 
                τ1.atom.final_state = τ2.atom.final_state
  potential_stable : S.V τ1.atom.initial_state = S.V τ2.atom.initial_state ∧
                     S.V τ1.atom.final_state = S.V τ2.atom.final_state
  budget_stable : 
    τ1.atom.cumulative_defect + τ1.atom.cumulative_authority - τ1.atom.cumulative_spend =
    τ2.atom.cumulative_defect + τ2.atom.cumulative_authority - τ2.atom.cumulative_spend

end Coh.Boundary
