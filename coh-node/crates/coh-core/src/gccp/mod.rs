//! GCCP - Governed Compute Control Plane
//!
//! A compute-specialized Coh object for compute dispatch with thermo-aware admission and
//! coherence-aware scheduling.
//!
//! # Key Components
//!
//! - [`state`](state) - Full and reduced state representations
//! - [`actions`](actions) - Typed action space
//! - [`pressure`](pressure) - System pressure functional V(x)
//! - [`spend`](spend) - Spend/Defect functionals
//! - [`guards`](guards) - Hard guard predicates
//!
//! # Usage
//!
//! ```rust
//! use coh_core::gccp::{GccpState, GccpAction, GccpActionDiscrete};
//! use coh_core::gccp::state::{ThermalState, PowerState, QueueState};
//!
//! // Create a minimal state
//! let state = GccpState::default();
//!
//! // Create a dispatch action
//! let action = GccpAction::Dispatch(coh_core::gccp::actions::DispatchAction::new(
//!     "matmul_kernel".to_string(),
//!     "GPU0".to_string(),
//!     "stream0".to_string(),
//!     1024,
//!     128,
//! ));
//! ```

pub mod actions;
pub mod admissibility;
pub mod guards;
pub mod predictor;
pub mod pressure;
pub mod spend;
pub mod state;

// Re-export key types for convenience
pub use actions::{
    CooldownAction, DeferAction, DispatchAction, GccpAction, GccpActionDiscrete, RejectAction,
    RerouteAction, ResizeAction,
};
pub use admissibility::{
    check_admissibility, check_admissibility_discrete, check_budget_boundedness,
    AdmissibilityResult,
};
pub use guards::{check_hard_guards, GuardResult, PolicyThresholds};
pub use predictor::{predict_next_state, predict_reduced, PredictedDelta};
pub use pressure::{
    compute_pressure, compute_pressure_reduced, NormalPolicy, PressureConfig, ThrottledPolicy,
};
pub use spend::{
    compute_defect, compute_defect_reduced, compute_spend, compute_spend_reduced, DefectConfig,
    SpendConfig,
};
pub use state::{
    BudgetState, ControlContext, GccpState, GccpStateReduced, MemoryState, OperatingMode,
    PowerState, QueueState, RiskState, ThermalState, UtilizationState, WorkloadClass,
};

/// GCCP module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gccp_state_creation() {
        let state = GccpState::default();
        assert_eq!(state.thermal.t_die, 45000);
    }

    #[test]
    fn test_gccp_action_dispatch() {
        let action = GccpAction::Dispatch(DispatchAction::new(
            "kernel1".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            512,
            100,
        ));
        assert_eq!(action.kernel(), Some("kernel1"));
    }

    #[test]
    fn test_discrete_action_costs() {
        use actions::GccpActionDiscrete;
        assert_eq!(GccpActionDiscrete::Heavy.energy_cost(), 150);
    }

    #[test]
    fn test_pressure_reduced() {
        let p = compute_pressure_reduced(50000, 100, 10);
        assert_eq!(p, 100110);
    }

    #[test]
    fn test_spend_reduced() {
        let s = compute_spend_reduced(GccpActionDiscrete::Heavy);
        assert_eq!(s, 150);
    }

    #[test]
    fn test_defect_reduced() {
        let d = compute_defect_reduced();
        assert_eq!(d, 500);
    }

    #[test]
    fn test_guard_pass() {
        let state = GccpState::default();
        let thresholds = PolicyThresholds::default();
        let result = check_hard_guards(&state, &thresholds, 1000);
        assert!(result.passed);
    }
}
