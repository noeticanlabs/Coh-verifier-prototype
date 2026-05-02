import Mathlib
import Mathlib.Analysis.Calculus.Fderiv
import Mathlib.Analysis.SpecialFunctions.Log.Basic
import Coh.Templates
import Coh.Boundary.CohBit

namespace Coh.Physics.Mechanics

/--
## Classical Mechanics State for CohBit
Maps directly to phase space (T*Q) where:
- q : Configuration (position)
- p : Momentum (cotangent vector)

This is the minimal state that carries both position and momentum
for Hamiltonian dynamics.
-/
structure State (Q : Type) (P : Type) where
  q : Q  -- configuration
  p : P  -- momentum

/--
## Classical Mechanics Transition
Represents a trajectory segment in phase space:
x₀ → x₁ with energy budget parameters.
-/
structure Transition (Q P : Type) where
  x₀ : State Q P    -- initial state
  x₁ : State Q P    -- final state
  spend : ℝ         -- dissipation/work done BY the system
  defect : ℝ        -- allowed perturbation/tolerance
  authority : ℝ      -- external work done ON the system

/--
## Trajectory (Chain)
A finite sequence of transitions forming an executable path.
-/
structure Trajectory (Q P : Type) where
  transitions : List (Transition Q P)
  continuous : transitions ≠ []  -- nonempty
  connection : ∀ i, (transitions.get? i).map (·.x₁) = (transitions.get? (i+1)).map (·.x₀)

/--
## Velocity from State Derivative
For a differentiable path q(t), velocity is dq/dt.
This is the natural projection from q to tangent space.
-/
def velocity {Q : Type} [NormedSpace ℝ Q] (q : ℝ → Q) (t : ℝ) : Q :=
  (stdDifferentiable 𝕜 q).dfdifferentiableWithinAt (Set.univ : Set ℝ) t |>.deriv

/--
## Configuration Space
A smooth manifold of positions.
-/
class ConfigSpace (Q : Type) where
  dimension : ℕ
  smooth_struct : SmoothManifoldWithCorners 𝓘 Q

/--
## Phase Space
Cotangent bundle T*Q where coordinates are (q,p).
-/
class PhaseSpace (Q P : Type) where
  config : ConfigSpace Q
  fiber : P ≃* (Q → ℝ)  -- trivialization

end Coh.Physics.Mechanics
