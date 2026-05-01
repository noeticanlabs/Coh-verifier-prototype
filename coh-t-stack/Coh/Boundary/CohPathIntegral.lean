import Mathlib
import Coh.Boundary.CohBit
import Coh.Boundary.CohAtom

namespace Coh.Boundary

/--
## Coh Path Integral (Admissible Histories)
\boxed{ \mathcal Z = \sum_{\{\mathcal H\}} e^{-\mathcal J[\mathcal H]/\tau} \prod_i \sigma(\beta m_i) }
-/

/--
### Coh History
A sequence of admissible bits forming a path through state space.
-/
structure CohHistory {X : Type} (S : CohSystem X) where
  steps : List (CohBit S)
  valid_sequence : steps.Chain' (fun a b => a.to_state = b.from_state)

/--
### History Action
The total action of a history is the sum of step actions.
-/
def history_action {X : Type} {S : CohSystem X} (h : CohHistory S) (lambda : ENNRat) : ENNRat :=
  h.steps.map (fun b => b.delta_hat -- Simplified: sum of defects as action
  ) |> List.sum

/--
### Finite-sample propagator over a list of histories
Aggregates the history_action over a provided finite list H.
-/
def propagator_over {X : Type} {S : CohSystem X}
  (H : List (CohHistory S)) (x_start x_end : X) (lambda : ENNRat) : ENNRat :=
  -- In this minimal formalization we ignore endpoint filtering and sum provided histories
  (H.map (fun h => history_action h lambda)).sum

end Coh.Boundary
