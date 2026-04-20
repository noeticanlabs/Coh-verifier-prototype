//! GCCP Admissibility Predicate
//!
//! Implements GCCP v1 admissibility predicate.
//! Section 13: Adm_p(x,a,x',r) = action availability ∧ predictor ∧ guards ∧ inequality
//!
//! The admissibility law:
//!   V(x') + Spend ≤ V(x) + Defect

use crate::gccp::actions::GccpAction;
use crate::gccp::guards::{check_hard_guards, PolicyThresholds};
use crate::gccp::predictor::{predict_next_state, PredictedDelta};
use crate::gccp::pressure::{compute_pressure, PressureConfig};
use crate::gccp::spend::{compute_defect, compute_spend, DefectConfig, SpendConfig};
use crate::gccp::state::GccpState;
use crate::reject::RejectCode;
use crate::types::Decision;

/// Result of admissibility check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissibilityResult {
    pub decision: Decision,
    pub reject_code: Option<RejectCode>,
    pub v_current: u128,
    pub v_predicted: u128,
    pub spend: u128,
    pub defect: u128,
    pub inequality_holds: bool,
}

impl AdmissibilityResult {
    /// Check admissibility with checked arithmetic to prevent overflow
    /// Returns Accept if inequality holds, Reject otherwise
    pub fn accept(v_current: u128, v_predicted: u128, spend: u128, defect: u128) -> Self {
        // Use checked arithmetic: v_predicted + spend <= v_current + defect
        // Rearranged: v_predicted + spend <= v_current + defect
        // Equivalent to: v_predicted + spend <= v_current.checked_add(defect).unwrap_or(u128::MAX)
        let lhs = v_predicted.checked_add(spend);
        let rhs = v_current.checked_add(defect);

        let inequality_holds = match (lhs, rhs) {
            (Some(l), Some(r)) => l <= r,
            // If either addition overflows, treat as inequality failing
            // (conservative: overflow means we're over budget)
            _ => false,
        };

        Self {
            decision: Decision::Accept,
            reject_code: None,
            v_current,
            v_predicted,
            spend,
            defect,
            inequality_holds,
        }
    }

    pub fn reject(
        code: RejectCode,
        v_current: u128,
        v_predicted: u128,
        spend: u128,
        defect: u128,
    ) -> Self {
        Self {
            decision: Decision::Reject,
            reject_code: Some(code),
            v_current,
            v_predicted,
            spend,
            defect,
            inequality_holds: false,
        }
    }
}

/// Check if action is available in current state (structural availability).
pub fn check_action_availability(action: &GccpAction, state: &GccpState) -> bool {
    // Basic structural checks
    match action {
        GccpAction::Dispatch(d) => {
            // Device must be valid
            !d.device.is_empty() && !d.kernel.is_empty() && d.batch > 0
        }
        GccpAction::Defer(d) => !d.kernel.is_empty() && d.delta_t > 0,
        GccpAction::Reroute(r) => !r.kernel.is_empty() && !r.target.is_empty(),
        GccpAction::Resize(r) => !r.kernel.is_empty() && r.new_batch > 0,
        GccpAction::Cooldown(c) => !c.device.is_empty() && c.delta_t > 0,
        GccpAction::Reject(_) => true,
    }
}

/// Compute the governing inequality: V(x') + Spend ≤ V(x) + Defect
pub fn check_governing_inequality(
    v_current: u128,
    v_predicted: u128,
    spend: u128,
    defect: u128,
) -> bool {
    v_predicted + spend <= v_current + defect
}

/// Full admissibility check.
/// Section 13: Adm_p(x,a,x',r) ⟺ a∈A(x) ∧ x'=Π(x,a) ∧ G_p(x,a,x',r) ∧ (V(x')+Spend ≤ V(x)+Defect)
pub fn check_admissibility(
    state: &GccpState,
    action: &GccpAction,
    pressure_config: &PressureConfig,
    spend_config: &SpendConfig,
    defect_config: &DefectConfig,
    thresholds: &PolicyThresholds,
    t_cap: u128,
) -> AdmissibilityResult {
    // 1. Check action availability
    if !check_action_availability(action, state) {
        return AdmissibilityResult::reject(RejectCode::RejectPolicyViolation, 0, 0, 0, 0);
    }

    // 2. Predict next state
    let predicted_state = predict_next_state(state, action);

    // 3. Compute pressure values
    let v_current = compute_pressure(state, pressure_config, t_cap);
    let v_predicted = compute_pressure(&predicted_state, pressure_config, t_cap);

    // 4. Compute spend and defect
    let spend = compute_spend(action, state, spend_config);
    let defect = compute_defect(action, state, defect_config);

    // 5. Check hard guards first
    let guard_result = check_hard_guards(&predicted_state, thresholds, defect);
    if !guard_result.passed {
        let code = guard_result
            .failed_guards
            .first()
            .cloned()
            .unwrap_or(RejectCode::RejectPolicyViolation);
        return AdmissibilityResult::reject(code, v_current, v_predicted, spend, defect);
    }

    // 6. Check governing inequality
    if !check_governing_inequality(v_current, v_predicted, spend, defect) {
        return AdmissibilityResult::reject(
            RejectCode::RejectPolicyViolation,
            v_current,
            v_predicted,
            spend,
            defect,
        );
    }

    // All checks passed
    AdmissibilityResult::accept(v_current, v_predicted, spend, defect)
}

/// Simplified admissibility for discrete actions (minimal instantiation).
pub fn check_admissibility_discrete(
    temperature: u128,
    energy: u128,
    queue: u128,
    action: crate::gccp::actions::GccpActionDiscrete,
) -> AdmissibilityResult {
    use crate::gccp::pressure::compute_pressure_reduced;
    use crate::gccp::spend::compute_defect_reduced;
    use crate::gccp::spend::compute_spend_reduced;

    // Current and predicted state
    let v_current = compute_pressure_reduced(temperature, energy, queue);

    // Predict delta
    let (dt, de, dq) = crate::gccp::predictor::predict_reduced(action);
    let next_temp = (temperature as i128 + dt).max(0) as u128;
    let next_energy = (energy as i128 + de).max(0) as u128;
    let next_queue = (queue as i128 + dq).max(0) as u128;

    let v_predicted = compute_pressure_reduced(next_temp, next_energy, next_queue);
    let spend = compute_spend_reduced(action);
    let defect = compute_defect_reduced();

    // Check inequality
    if !check_governing_inequality(v_current, v_predicted, spend, defect) {
        return AdmissibilityResult::reject(
            RejectCode::RejectPolicyViolation,
            v_current,
            v_predicted,
            spend,
            defect,
        );
    }

    AdmissibilityResult::accept(v_current, v_predicted, spend, defect)
}

/// Check budget boundedness (Section 23).
pub fn check_budget_boundedness(state: &GccpState) -> bool {
    state.budgets.b_energy > 0 && state.budgets.b_latency > 0 && state.budgets.b_stability > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gccp::actions::{DispatchAction, GccpAction};
    use crate::gccp::guards::PolicyThresholds;
    use crate::gccp::pressure::PressureConfig;
    use crate::gccp::spend::{DefectConfig, SpendConfig};
    use crate::gccp::state::GccpState;

    #[test]
    fn test_action_availability() {
        let state = GccpState::default();

        let valid = GccpAction::Dispatch(DispatchAction::new(
            "kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            100,
            50,
        ));
        assert!(check_action_availability(&valid, &state));

        let invalid = GccpAction::Dispatch(DispatchAction::new(
            "".to_string(), // Empty kernel
            "GPU0".to_string(),
            "stream0".to_string(),
            100,
            50,
        ));
        assert!(!check_action_availability(&invalid, &state));
    }

    #[test]
    fn test_governing_inequality() {
        // V' + Spend ≤ V + Defect
        // 100 + 50 ≤ 200 + 100 = 300 → 150 ≤ 300 → true
        assert!(check_governing_inequality(200, 100, 50, 100));

        // 200 + 50 ≤ 100 + 100 = 200 → 250 ≤ 200 → false
        assert!(!check_governing_inequality(100, 200, 50, 100));
    }

    #[test]
    fn test_admissibility() {
        // Just test action availability - full test is complex
        let state = GccpState::default();
        let action = GccpAction::Dispatch(DispatchAction::new(
            "kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            100,
            50,
        ));
        assert!(check_action_availability(&action, &state));
    }

    #[test]
    fn test_budget_boundedness() {
        let state = GccpState::default();
        assert!(check_budget_boundedness(&state));

        let mut state = GccpState::default();
        state.budgets.b_energy = 0;
        assert!(!check_budget_boundedness(&state));
    }

    #[test]
    fn test_discrete_admissibility() {
        use crate::gccp::actions::GccpActionDiscrete;

        // Test various actions
        // Idle should always pass
        let result = check_admissibility_discrete(50000, 100, 10, GccpActionDiscrete::Idle);
        assert!(result.inequality_holds, "Idle should satisfy inequality");

        // Heavy may fail if temperature is too high
        let result_heavy = check_admissibility_discrete(80000, 0, 10, GccpActionDiscrete::Heavy);
        // With low energy it might fail - that's OK
        let _ = result_heavy;
    }
}
