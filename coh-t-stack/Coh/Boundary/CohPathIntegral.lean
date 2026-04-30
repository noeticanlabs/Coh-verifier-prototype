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
### Propagator
K(x_0, x_n) = \sum_{H: x_0 \to x_n} e^{-\mathcal J(H)/\tau}
Represents the total probability flow between two states.
-/
def propagator {X : Type} {S : CohSystem X} (x_start x_end : X) (tau : ENNRat) : ENNRat :=
  sorry -- Sum over all valid histories from x_start to x_end

end Coh.Boundary
