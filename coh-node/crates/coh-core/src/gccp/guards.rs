//! GCCP Hard Guards
//!
//! Implements GCCP v1 hard guard predicates.
//! Section 11: Hard guards before admissibility checking.
//!
//! G_p(x,a,ẋ',r) checks:
//!   - Thot ≤ Tmax(p)
//!   - Pnow ≤ Pmax(p)
//!   - q_age ≤ Qmax(p)
//!   - m_used ≤ Mmax(p)
//!   - r_throttle ≤ Rmax(p)
//!   - Defect* ≤ δp
//!   - bE ≥ Emin(p), bL ≥ Lmin(p), bS ≥ Smin(p)

use crate::gccp::spend::DefectConfig;
use crate::gccp::state::{BudgetState, GccpState, MemoryState, QueueState, RiskState};
use crate::reject::RejectCode;

/// Hard guard result - which guards failed
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardResult {
    pub passed: bool,
    pub failed_guards: Vec<RejectCode>,
}

impl GuardResult {
    pub fn pass() -> Self {
        Self {
            passed: true,
            failed_guards: vec![],
        }
    }

    pub fn fail(code: RejectCode) -> Self {
        Self {
            passed: false,
            failed_guards: vec![code],
        }
    }

    pub fn add_fail(&mut self, code: RejectCode) {
        self.passed = false;
        self.failed_guards.push(code);
    }
}

/// Policy thresholds for hard guards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyThresholds {
    pub t_max: u128,
    pub p_max: u128,
    pub q_age_max: u128,
    pub m_used_max: u128,
    pub r_throttle_max: u128,
    pub defect_max: u128,
    pub e_min: u128,
    pub l_min: u128,
    pub s_min: u128,
}

impl Default for PolicyThresholds {
    fn default() -> Self {
        Self {
            t_max: 90000,       // 90°C
            p_max: 300000,      // 300W
            q_age_max: 10000,   // 10s age
            m_used_max: 16000,  // 16GB
            r_throttle_max: 10, // 10 throttle events
            defect_max: 5000,   // 5 unit slack
            e_min: 1000,        // 1kJ
            l_min: 10000,       // 10s
            s_min: 100,         // 100 stability budget
        }
    }
}

/// Check thermal guard: Thot ≤ Tmax(p)
pub fn check_thermal_guard(state: &GccpState, thresholds: &PolicyThresholds) -> GuardResult {
    if state.thermal.t_hot > thresholds.t_max {
        GuardResult::fail(RejectCode::RejectTempCap)
    } else {
        GuardResult::pass()
    }
}

/// Check power guard: Pnow ≤ Pmax(p)
pub fn check_power_guard(state: &GccpState, thresholds: &PolicyThresholds) -> GuardResult {
    if state.power.p_now > thresholds.p_max {
        GuardResult::fail(RejectCode::RejectPowerCap)
    } else {
        GuardResult::pass()
    }
}

/// Check queue age guard: q_age ≤ Qmax(p)
pub fn check_queue_guard(state: &GccpState, thresholds: &PolicyThresholds) -> GuardResult {
    if state.queue.q_age > thresholds.q_age_max {
        GuardResult::fail(RejectCode::RejectQueueCap)
    } else {
        GuardResult::pass()
    }
}

/// Check memory guard: m_used ≤ Mmax(p)
pub fn check_memory_guard(state: &GccpState, thresholds: &PolicyThresholds) -> GuardResult {
    if state.memory.m_used > thresholds.m_used_max {
        GuardResult::fail(RejectCode::RejectMemoryCap)
    } else {
        GuardResult::pass()
    }
}

/// Check throttle guard: r_throttle ≤ Rmax(p)
pub fn check_throttle_guard(state: &GccpState, thresholds: &PolicyThresholds) -> GuardResult {
    if state.risk.r_throttle > thresholds.r_throttle_max {
        GuardResult::fail(RejectCode::RejectDefectCap)
    } else {
        GuardResult::pass()
    }
}

/// Check defect/slack budget: Defect* ≤ δp
pub fn check_defect_guard(defect_value: u128, thresholds: &PolicyThresholds) -> GuardResult {
    if defect_value > thresholds.defect_max {
        GuardResult::fail(RejectCode::RejectDefectCap)
    } else {
        GuardResult::pass()
    }
}

/// Check budget guards: bE ≥ Emin, bL ≥ Lmin, bS ≥ Smin
pub fn check_budget_guard(state: &BudgetState, thresholds: &PolicyThresholds) -> GuardResult {
    let mut result = GuardResult::pass();

    if state.b_energy < thresholds.e_min {
        result.add_fail(RejectCode::RejectBudget);
    }
    if state.b_latency < thresholds.l_min {
        result.add_fail(RejectCode::RejectBudget);
    }
    if state.b_stability < thresholds.s_min {
        result.add_fail(RejectCode::RejectBudget);
    }

    result
}

/// Compute predicted next state and check all hard guards.
/// Returns the guard result and predicted next state (if guards pass).
pub fn check_hard_guards(
    state: &GccpState,
    thresholds: &PolicyThresholds,
    defect_value: u128,
) -> GuardResult {
    let mut result = GuardResult::pass();

    // Check each guard and accumulate failures
    let thermal = check_thermal_guard(state, thresholds);
    if !thermal.passed {
        result.add_fail(RejectCode::RejectTempCap);
    }

    let power = check_power_guard(state, thresholds);
    if !power.passed {
        result.add_fail(RejectCode::RejectPowerCap);
    }

    let queue = check_queue_guard(state, thresholds);
    if !queue.passed {
        result.add_fail(RejectCode::RejectQueueCap);
    }

    let memory = check_memory_guard(state, thresholds);
    if !memory.passed {
        result.add_fail(RejectCode::RejectMemoryCap);
    }

    let throttle = check_throttle_guard(state, thresholds);
    if !throttle.passed {
        result.add_fail(RejectCode::RejectDefectCap);
    }

    let defect = check_defect_guard(defect_value, thresholds);
    if !defect.passed {
        result.add_fail(RejectCode::RejectDefectCap);
    }

    let budgets = check_budget_guard(&state.budgets, thresholds);
    if !budgets.passed {
        // Budget already has the code
        for code in budgets.failed_guards {
            result.add_fail(code);
        }
    }

    result
}

/// Check hard guards for predicted next state (post-action).
/// This version checks predicted values, not current state.
pub fn check_predicted_hard_guards(
    predicted_state: &GccpState,
    thresholds: &PolicyThresholds,
    predicted_defect: u128,
) -> GuardResult {
    check_hard_guards(predicted_state, thresholds, predicted_defect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gccp::state::{BudgetState, GccpState, PowerState, ThermalState};

    #[test]
    fn test_thermal_guard_pass() {
        let state = GccpState::default();
        let thresholds = PolicyThresholds::default();

        let result = check_thermal_guard(&state, &thresholds);
        assert!(result.passed);
    }

    #[test]
    fn test_thermal_guard_fail() {
        let mut state = GccpState::default();
        state.thermal.t_hot = 100000; // Above 90°C threshold

        let thresholds = PolicyThresholds::default();
        let result = check_thermal_guard(&state, &thresholds);

        assert!(!result.passed);
        assert!(result.failed_guards.contains(&RejectCode::RejectTempCap));
    }

    #[test]
    fn test_power_guard_pass() {
        let state = GccpState::default();
        let thresholds = PolicyThresholds::default();

        let result = check_power_guard(&state, &thresholds);
        assert!(result.passed);
    }

    #[test]
    fn test_budget_guard_pass() {
        let state = GccpState::default();
        let thresholds = PolicyThresholds::default();

        let result = check_budget_guard(&state.budgets, &thresholds);
        assert!(result.passed);
    }

    #[test]
    fn test_budget_guard_fail() {
        let mut state = GccpState::default();
        state.budgets.b_energy = 0; // Zero energy

        let thresholds = PolicyThresholds::default();
        let result = check_budget_guard(&state.budgets, &thresholds);

        assert!(!result.passed);
    }

    #[test]
    fn test_defect_guard() {
        let thresholds = PolicyThresholds::default();

        // Under limit
        let result = check_defect_guard(1000, &thresholds);
        assert!(result.passed);

        // Over limit
        let result = check_defect_guard(10000, &thresholds);
        assert!(!result.passed);
    }

    #[test]
    fn test_combined_guards() {
        let state = GccpState::default();
        let thresholds = PolicyThresholds::default();

        let result = check_hard_guards(&state, &thresholds, 1000);
        assert!(result.passed);
    }
}
