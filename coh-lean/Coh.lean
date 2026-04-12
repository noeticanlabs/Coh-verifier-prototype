import Coh.Prelude
import Coh.Core.Clifford
import Coh.Core.Oplax
import Coh.Core.Minimality
import Coh.Core.Complexification
import Coh.Core.Chain
import Coh.Core.Dynamics
import Coh.Core.CliffordRep
import Coh.Examples.DiracMatrixWitness
import Coh.Kinematics.T3_CoerciveVisibility
import Coh.Kinematics.T3_Necessity
import Coh.Kinematics.T3_NonCliffordVisible
import Coh.Kinematics.T3_WitnessAmplification
import Coh.Thermo.T5_Minimality
import Coh.Thermo.T5_RepresentationMinimality
import Coh.Geometry.T6_Complexification
import Coh.Geometry.T6_PersistenceForcesRotation
import Coh.Geometry.T6_CommutesWithClifford
import Coh.Spectral.AnomalyStrength
import Coh.Spectral.VisibilityGap
import Coh.Spectral.CompactnessProof
import Coh.Spectral.DefectAccumulation
import Coh.Spectral.T8_StabilityMinimality
import Coh.Spectral.T9_GaugeEmergence
import Coh.Spectral.T10_DiracDynamics
import Coh.Physics.DiracInevitable

/-!
# Coh Safety Kernel: Formal Lean 4 Scaffold

This is the umbrella module for the Coh-Fusion safety kernel formalization.
The project is structured into three primary theorem stacks:

- **T3 (Kinematics)**: Clifford necessity from measurement soundness.
- **T5 (Thermodynamics)**: Representation minimality from metabolic cost.
- **T6 (Geometry)**: Complex structure from stable periodicity.

Each stack is modularized into:
1.  **Core Definitions**: The proved algebraic properties.
2.  **Analysis Bridges**: Asymptotic and reduction layers.
3.  **Necessity Layers**: The final converse/selection logic.

This scaffold represents the 100% formalized baseline, with exact analytic
obligations identified as bridges (schemas) for final discharge.
-/

namespace Coh

-- Root namespace for the Coh safety kernel.

end Coh
