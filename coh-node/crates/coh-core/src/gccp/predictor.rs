//! GCCP Predictor
//!
//! Implements GCCP v1 state predictor Π: X × A → X.
//! Given current state and action, predicts the next state.
//!
//! Section 7 of GCCP v1 specification.

use crate::gccp::actions::GccpAction;
use crate::gccp::state::{
    BudgetState, GccpState, MemoryState, PowerState, QueueState, RiskState, ThermalState,
    UtilizationState,
};

/// Predicted state delta from an action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredictedDelta {
    pub thermal_delta: i128,
    pub power_delta: i128,
    pub queue_delta: i128,
    pub memory_delta: i128,
    pub risk_delta: i128,
    pub energy_cost: u128,
    pub latency_cost: u128,
}

impl Default for PredictedDelta {
    fn default() -> Self {
        Self {
            thermal_delta: 0,
            power_delta: 0,
            queue_delta: 0,
            memory_delta: 0,
            risk_delta: 0,
            energy_cost: 0,
            latency_cost: 0,
        }
    }
}

/// Predict state delta from a dispatch action.
pub fn predict_dispatch(action: &GccpAction) -> PredictedDelta {
    use crate::gccp::actions::DispatchAction;

    match action {
        GccpAction::Dispatch(d) => {
            let batch = d.batch;
            PredictedDelta {
                // Thermal rises with batch size and GPU usage
                thermal_delta: if d.device.starts_with("GPU") {
                    batch as i128 * 5
                } else {
                    batch as i128 * 2
                },
                // Power scales with batch
                power_delta: if d.device.starts_with("GPU") {
                    batch / 10
                } else {
                    batch / 20
                } as i128,
                // Queue increases during dispatch
                queue_delta: batch as i128 / 100,
                // Memory proportional to batch
                memory_delta: batch as i128 * 4,
                // Risk from execution
                risk_delta: 0,
                // Energy consumed
                energy_cost: if d.device.starts_with("GPU") {
                    batch / 10
                } else {
                    batch / 20
                },
                // Latency proportional
                latency_cost: batch * 10,
            }
        }
        _ => PredictedDelta::default(),
    }
}

/// Predict state delta from a defer action.
pub fn predict_defer(action: &GccpAction) -> PredictedDelta {
    use crate::gccp::actions::DeferAction;

    match action {
        GccpAction::Defer(d) => {
            // Age increases, but no resource usage
            PredictedDelta {
                thermal_delta: 0,
                power_delta: 0,
                queue_delta: 0,
                memory_delta: 0,
                risk_delta: (d.delta_t / 1000) as i128, // Age pressure from waiting
                energy_cost: 0,
                latency_cost: d.delta_t,
            }
        }
        _ => PredictedDelta::default(),
    }
}

/// Predict state delta from a reroute action.
pub fn predict_reroute(action: &GccpAction) -> PredictedDelta {
    use crate::gccp::actions::RerouteAction;

    match action {
        GccpAction::Reroute(_) => {
            // Small overhead for rerouting
            PredictedDelta {
                thermal_delta: -500, // Cooling from idle
                power_delta: -1000,
                queue_delta: 0,
                memory_delta: 0,
                risk_delta: 1, // Retry risk
                energy_cost: 10,
                latency_cost: 100,
            }
        }
        _ => PredictedDelta::default(),
    }
}

/// Predict state delta from a resize action.
pub fn predict_resize(action: &GccpAction) -> PredictedDelta {
    use crate::gccp::actions::ResizeAction;

    match action {
        GccpAction::Resize(r) => {
            // New batch size
            PredictedDelta {
                thermal_delta: -(r.new_batch as i128 * 2), // Cooling effect
                power_delta: -(r.new_batch as i128 / 10),
                queue_delta: 0,
                memory_delta: -(r.new_batch as i128 * 2),
                risk_delta: 0,
                energy_cost: 5,
                latency_cost: 50,
            }
        }
        _ => PredictedDelta::default(),
    }
}

/// Predict state delta from a cooldown action.
pub fn predict_cooldown(action: &GccpAction) -> PredictedDelta {
    use crate::gccp::actions::CooldownAction;

    match action {
        GccpAction::Cooldown(c) => {
            // Active cooling
            PredictedDelta {
                thermal_delta: -((c.delta_t / 100) as i128).min(5000), // Cap cooling
                power_delta: -1000,
                queue_delta: 0,
                memory_delta: 0,
                risk_delta: 0,
                energy_cost: c.delta_t / 100, // Small energy for cooling
                latency_cost: c.delta_t,
            }
        }
        _ => PredictedDelta::default(),
    }
}

/// Predict state delta from a reject action (clears pending work).
pub fn predict_reject(action: &GccpAction) -> PredictedDelta {
    use crate::gccp::actions::RejectAction;

    match action {
        GccpAction::Reject(_) => {
            // Risk increases from rejection, clears queue
            PredictedDelta {
                thermal_delta: -1000,
                power_delta: -1000,
                queue_delta: -1, // Clears entry
                memory_delta: 0,
                risk_delta: 2, // Failure risk
                energy_cost: 0,
                latency_cost: 0,
            }
        }
        _ => PredictedDelta::default(),
    }
}

/// Predict next state from current state and action.
/// Π(x, a) → x'
pub fn predict_next_state(state: &GccpState, action: &GccpAction) -> GccpState {
    let delta = predict_action_delta(action);

    let mut next = state.clone();

    // Apply thermal delta
    next.thermal.t_die = (next.thermal.t_die as i128 + delta.thermal_delta).max(0) as u128;
    next.thermal.t_hot = (next.thermal.t_hot as i128 + delta.thermal_delta).max(0) as u128;
    next.thermal.t_rise = delta.thermal_delta;

    // Apply power delta
    next.power.p_now = (next.power.p_now as i128 + delta.power_delta).max(0) as u128;
    next.power.p_margin = next.power.p_cap.saturating_sub(next.power.p_now);

    // Apply queue delta
    next.queue.q_depth = (next.queue.q_depth as i128 + delta.queue_delta).max(0) as u128;
    next.queue.q_age =
        (next.queue.q_age as i128 + delta.latency_cost as i128).min(u128::MAX as i128) as u128;

    // Apply memory delta
    next.memory.m_used = (next.memory.m_used as i128 + delta.memory_delta).max(0) as u128;

    // Apply risk delta
    next.risk.r_retry =
        (next.risk.r_retry as i128 + delta.risk_delta).min(u128::MAX as i128) as u128;

    // Deduct energy and latency budgets
    next.budgets.b_energy = next.budgets.b_energy.saturating_sub(delta.energy_cost);
    next.budgets.b_latency = next.budgets.b_latency.saturating_sub(delta.latency_cost);

    next
}

/// Combined delta prediction for any action.
pub fn predict_action_delta(action: &GccpAction) -> PredictedDelta {
    match action {
        GccpAction::Dispatch(_) => predict_dispatch(action),
        GccpAction::Defer(_) => predict_defer(action),
        GccpAction::Reroute(_) => predict_reroute(action),
        GccpAction::Resize(_) => predict_resize(action),
        GccpAction::Cooldown(_) => predict_cooldown(action),
        GccpAction::Reject(_) => predict_reject(action),
    }
}

/// Simple predictor for reduced state (T, E, Q).
pub fn predict_reduced(action: crate::gccp::actions::GccpActionDiscrete) -> (i128, i128, i128) {
    // Returns (delta_temp, delta_energy, delta_queue)
    let thermal = action.thermal_rise() as i128;
    let energy = action.energy_cost() as i128;
    let queue = action.queue_impact() as i128;
    (thermal, energy, queue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gccp::actions::{DispatchAction, GccpAction};
    use crate::gccp::state::GccpState;

    #[test]
    fn test_predict_dispatch() {
        let action = GccpAction::Dispatch(DispatchAction::new(
            "kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            1000,
            100,
        ));
        let delta = predict_dispatch(&action);
        assert!(delta.thermal_delta > 0);
        assert!(delta.energy_cost > 0);
    }

    #[test]
    fn test_predict_cooldown() {
        let action = GccpAction::Cooldown(crate::gccp::actions::CooldownAction::new(
            "GPU0".to_string(),
            5000,
        ));
        let delta = predict_cooldown(&action);
        assert!(delta.thermal_delta < 0); // Cooling
    }

    #[test]
    fn test_predict_next_state() {
        let state = GccpState::default();
        let action = GccpAction::Dispatch(DispatchAction::new(
            "kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            1000,
            100,
        ));

        let next = predict_next_state(&state, &action);

        // State should have changed
        assert_ne!(next.thermal.t_die, state.thermal.t_die);
    }

    #[test]
    fn test_reduced_predictor() {
        use crate::gccp::actions::GccpActionDiscrete;

        let (dt, de, dq) = predict_reduced(GccpActionDiscrete::Heavy);
        assert_eq!(dt, 20000);
        assert_eq!(de, 150);
        assert_eq!(dq, 10);
    }
}
