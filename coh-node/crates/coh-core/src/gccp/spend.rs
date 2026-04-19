//! GCCP Spend and Defect Functionals
//!
//! Implements GCCP v1 Spend and Defect functionals.
//!
//! Spend (Section 9): predicted irreversible cost
//!   Spend = αE·E + αT·ΔT + αQ·ΔQ+ + αL·L + αW·W
//!
//! Defect (Section 10): bounded uncertainty/slack
//!   Defect = d_pred + d_tele + d_model + d_quant
//!
//! Section 9-10 of GCCP v1 specification.

use crate::gccp::actions::GccpAction;
use crate::gccp::state::GccpState;

/// Spend functional coefficients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendConfig {
    pub alpha_energy: u128,  // αE
    pub alpha_thermal: u128, // αT
    pub alpha_queue: u128,   // αQ
    pub alpha_latency: u128, // αL
    pub alpha_wear: u128,    // αW
}

impl Default for SpendConfig {
    fn default() -> Self {
        Self {
            alpha_energy: 100,
            alpha_thermal: 50,
            alpha_queue: 20,
            alpha_latency: 10,
            alpha_wear: 5,
        }
    }
}

/// Defect/slack functional coefficients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefectConfig {
    pub d_predictor: u128,    // d_pred - predictor error bound
    pub d_telemetry: u128,    // d_tele - telemetry staleness
    pub d_model: u128,        // d_model - coarse model mismatch
    pub d_quantization: u128, // d_quant - fixed-point slack
}

impl Default for DefectConfig {
    fn default() -> Self {
        Self {
            d_predictor: 1000,
            d_telemetry: 500,
            d_model: 2000,
            d_quantization: 100,
        }
    }
}

/// Predicted energy consumption for an action (in joules).
pub fn predict_energy(action: &GccpAction, current_state: &GccpState) -> u128 {
    use crate::gccp::actions::GccpActionDiscrete;

    match action {
        GccpAction::Dispatch(d) => {
            // Energy scales with batch size
            let base = if d.device.starts_with("GPU") { 50 } else { 20 };
            base + d.batch / 10
        }
        GccpAction::Defer(_) => 0,
        GccpAction::Reroute(_) => 10,
        GccpAction::Resize(r) => {
            // Energy change from resize
            if r.new_batch > 0 {
                5
            } else {
                0
            }
        }
        GccpAction::Cooldown(_) => 0,
        GccpAction::Reject(_) => 0,
    }
}

/// Predicted thermal rise from an action (in millidegrees C).
pub fn predict_thermal_rise(action: &GccpAction, current_state: &GccpState) -> u128 {
    use crate::gccp::actions::GccpActionDiscrete;

    match action {
        GccpAction::Dispatch(d) => {
            // Heat scales with batch and device
            let base = if d.device.starts_with("GPU") {
                5000
            } else {
                2000
            };
            base + d.batch as u128 * 5
        }
        GccpAction::Defer(_) => 0,
        GccpAction::Reroute(_) => 1000,
        GccpAction::Resize(_) => 500,
        GccpAction::Cooldown(c) => {
            // Negative - cooling
            c.delta_t.min(10000) as u128
        }
        GccpAction::Reject(_) => 0,
    }
}

/// Predicted queue depth change (positive only).
pub fn predict_queue_delta(action: &GccpAction, current_state: &GccpState) -> u128 {
    use crate::gccp::actions::GccpActionDiscrete;

    match action {
        GccpAction::Dispatch(d) => {
            // Queue increases during dispatch
            d.batch / 100
        }
        GccpAction::Defer(_) => {
            // Defer doesn't change queue
            0
        }
        GccpAction::Reroute(_) => 0,
        GccpAction::Resize(_) => 0,
        GccpAction::Cooldown(_) => 0,
        GccpAction::Reject(r) => {
            // Reject clears queue entry
            1
        }
    }
}

/// Predicted latency budget consumption (in milliseconds).
pub fn predict_latency(action: &GccpAction, current_state: &GccpState) -> u128 {
    match action {
        GccpAction::Dispatch(d) => {
            // Rough latency estimate
            d.batch * 10
        }
        GccpAction::Defer(defer) => defer.delta_t,
        GccpAction::Reroute(_) => 100,
        GccpAction::Resize(_) => 50,
        GccpAction::Cooldown(c) => c.delta_t,
        GccpAction::Reject(_) => 0,
    }
}

/// Predicted wear/stress (cumulative degradation proxy).
pub fn predict_wear(action: &GccpAction, current_state: &GccpState) -> u128 {
    use crate::gccp::actions::GccpActionDiscrete;

    match action {
        GccpAction::Dispatch(d) => {
            // More batch = more wear
            d.batch / 500
        }
        GccpAction::Defer(_) => 0,
        GccpAction::Reroute(_) => 1,
        GccpAction::Resize(_) => 0,
        GccpAction::Cooldown(_) => 0,
        GccpAction::Reject(_) => 0,
    }
}

/// Compute Spend value.
/// Section 9: Spend = αE·E + αT·ΔT + αQ·ΔQ+ + αL·L + αW·W
pub fn compute_spend(action: &GccpAction, current_state: &GccpState, config: &SpendConfig) -> u128 {
    let e = predict_energy(action, current_state);
    let dt = predict_thermal_rise(action, current_state);
    let dq = predict_queue_delta(action, current_state);
    let l = predict_latency(action, current_state);
    let w = predict_wear(action, current_state);

    config.alpha_energy * e
        + config.alpha_thermal * dt
        + config.alpha_queue * dq
        + config.alpha_latency * l
        + config.alpha_wear * w
}

/// Compute Defect/slack value.
/// Section 10: Defect = d_pred + d_tele + d_model + d_quant
pub fn compute_defect(
    action: &GccpAction,
    current_state: &GccpState,
    config: &DefectConfig,
) -> u128 {
    // These can vary based on state (e.g., telemetry age)
    let state_age = current_state.context.policy_hash.len();

    // Telemetry increases with state age
    let telemetry_factor = (state_age as u128).min(10);

    config.d_predictor
        + config.d_telemetry * telemetry_factor
        + config.d_model
        + config.d_quantization
}

/// Reduced spend for minimal instantiation (T, E, Q).
pub fn compute_spend_reduced(action: crate::gccp::actions::GccpActionDiscrete) -> u128 {
    action.energy_cost()
}

/// Reduced defect (constant).
pub fn compute_defect_reduced() -> u128 {
    500 // Fixed defect for reduced model
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gccp::actions::{DispatchAction, GccpAction};
    use crate::gccp::state::GccpState;

    #[test]
    fn test_predict_energy() {
        let state = GccpState::default();

        let action = GccpAction::Dispatch(DispatchAction::new(
            "kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            1000,
            100,
        ));

        let energy = predict_energy(&action, &state);
        assert!(energy > 0);
    }

    #[test]
    fn test_predict_thermal_rise() {
        let state = GccpState::default();

        let action = GccpAction::Dispatch(DispatchAction::new(
            "kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            1000,
            100,
        ));

        let rise = predict_thermal_rise(&action, &state);
        assert!(rise > 0);
    }

    #[test]
    fn test_compute_spend() {
        let state = GccpState::default();
        let config = SpendConfig::default();

        let action = GccpAction::Dispatch(DispatchAction::new(
            "kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            1000,
            100,
        ));

        let spend = compute_spend(&action, &state, &config);
        assert!(spend > 0);
    }

    #[test]
    fn test_compute_defect() {
        let state = GccpState::default();
        let config = DefectConfig::default();

        let action = GccpAction::Dispatch(DispatchAction::new(
            "kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            1000,
            100,
        ));

        let defect = compute_defect(&action, &state, &config);
        assert!(defect > 0);
    }

    #[test]
    fn test_discrete_spend() {
        use crate::gccp::actions::GccpActionDiscrete;

        assert_eq!(GccpActionDiscrete::Heavy.energy_cost(), 150);
        assert_eq!(GccpActionDiscrete::Idle.energy_cost(), 0);
    }
}
