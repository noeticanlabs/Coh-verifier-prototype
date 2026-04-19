//! GCCP Pressure Functional
//!
//! Implements the GCCP v1 value functional V(x) = system pressure.
//! V(x) = wT·φT(x) + wP·φP(x) + wQ·φQ(x) + wM·φM(x) + wR·φR(x) + wB·φB(x)
//!
//! Section 8 of GCCP v1 specification.

use crate::gccp::state::{
    BudgetState, GccpState, MemoryState, PowerState, QueueState, RiskState, ThermalState,
    UtilizationState,
};

/// Policy configuration for pressure computation.
/// Weights and coefficients indexed by policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PressureConfig {
    /// Primary weights (positive)
    pub w_thermal: u128,
    pub w_power: u128,
    pub w_queue: u128,
    pub w_memory: u128,
    pub w_risk: u128,
    pub w_budget: u128,
    /// Thermal coefficients (λ1, λ2, λ3)
    pub lambda1: u128,
    pub lambda2: u128,
    pub lambda3: u128,
    /// Power coefficients (μ1, μ2)
    pub mu1: u128,
    pub mu2: u128,
    /// Queue coefficients (ν1, ν2, ν3)
    pub nu1: u128,
    pub nu2: u128,
    pub nu3: u128,
    /// Memory coefficients (ξ1, ξ2, ξ3)
    pub xi1: u128,
    pub xi2: u128,
    pub xi3: u128,
    /// Risk coefficients (ρ1, ρ2, ρ3)
    pub rho1: u128,
    pub rho2: u128,
    pub rho3: u128,
    /// Budget barrier coefficients (η1, η2, η3)
    pub eta1: u128,
    pub eta2: u128,
    pub eta3: u128,
    /// Small constant to prevent division by zero
    pub epsilon: u128,
}

impl Default for PressureConfig {
    fn default() -> Self {
        Self {
            w_thermal: 100,
            w_power: 100,
            w_queue: 50,
            w_memory: 50,
            w_risk: 200,
            w_budget: 300,
            // Thermal
            lambda1: 1,
            lambda2: 2,
            lambda3: 5,
            // Power
            mu1: 1,
            mu2: 3,
            // Queue
            nu1: 1,
            nu2: 2,
            nu3: 1,
            // Memory
            xi1: 1,
            xi2: 1,
            xi3: 2,
            // Risk
            rho1: 5,
            rho2: 5,
            rho3: 10,
            // Budget barrier
            eta1: 1000,
            eta2: 100,
            eta3: 100,
            // Epsilon
            epsilon: 1,
        }
    }
}

/// Thermal pressure: φT(x) = λ1·Tdie + λ2·Thot + λ3·(Thot - Tcap)+
/// Positive part: max(Thot - Tcap, 0)
pub fn thermal_pressure(state: &ThermalState, config: &PressureConfig, t_cap: u128) -> u128 {
    let t_hot_part = if state.t_hot > t_cap {
        state.t_hot - t_cap
    } else {
        0
    };
    config.lambda1 * state.t_die + config.lambda2 * state.t_hot + config.lambda3 * t_hot_part
}

/// Power pressure: φP(x) = μ1·Pnow + μ2·(Pnow - Pcap)+
pub fn power_pressure(state: &PowerState, config: &PressureConfig) -> u128 {
    let p_over = if state.p_now > state.p_cap {
        state.p_now - state.p_cap
    } else {
        0
    };
    config.mu1 * state.p_now + config.mu2 * p_over
}

/// Queue pressure: φQ(x) = ν1·qdepth + ν2·qage + ν3·qmix
pub fn queue_pressure(state: &QueueState, config: &PressureConfig) -> u128 {
    config.nu1.saturating_mul(state.q_depth)
        + config.nu2.saturating_mul(state.q_age)
        + config.nu3.saturating_mul(state.q_mix)
}

/// Memory pressure: φM(x) = ξ1·mused + ξ2·mbw + ξ3·mfrag
pub fn memory_pressure(state: &MemoryState, config: &PressureConfig) -> u128 {
    config.xi1.saturating_mul(state.m_used)
        + config.xi2.saturating_mul(state.m_bw)
        + config.xi3.saturating_mul(state.m_frag)
}

/// Instability/Risk pressure: φR(x) = ρ1·rretry + ρ2·rtimeout + ρ3·rthrottle
pub fn risk_pressure(state: &RiskState, config: &PressureConfig) -> u128 {
    config.rho1.saturating_mul(state.r_retry)
        + config.rho2.saturating_mul(state.r_timeout)
        + config.rho3.saturating_mul(state.r_throttle)
}

/// Budget barrier: φB(x) = η1/(bE+ε) + η2/(bL+ε) + η3/(bS+ε)
/// Returns u128::MAX if any budget is zero (infinite pressure)
pub fn budget_barrier(state: &BudgetState, config: &PressureConfig) -> u128 {
    let b_e = if state.b_energy == 0 {
        return u128::MAX;
    } else {
        state.b_energy
    };
    let b_l = if state.b_latency == 0 {
        return u128::MAX;
    } else {
        state.b_latency
    };
    let b_s = if state.b_stability == 0 {
        return u128::MAX;
    } else {
        state.b_stability
    };

    // Use saturating division approximation
    config.eta1 * (config.epsilon * 1000 / b_e)
        + config.eta2 * (config.epsilon * 1000 / b_l)
        + config.eta3 * (config.epsilon * 1000 / b_s)
}

/// Compute full pressure V(x) for state.
/// This is the system pressure functional from Section 8.
pub fn compute_pressure(state: &GccpState, config: &PressureConfig, t_thermal_cap: u128) -> u128 {
    let phi_t = thermal_pressure(&state.thermal, config, t_thermal_cap);
    let phi_p = power_pressure(&state.power, config);
    let phi_q = queue_pressure(&state.queue, config);
    let phi_m = memory_pressure(&state.memory, config);
    let phi_r = risk_pressure(&state.risk, config);
    let phi_b = budget_barrier(&state.budgets, config);

    config.w_thermal * phi_t
        + config.w_power * phi_p
        + config.w_queue * phi_q
        + config.w_memory * phi_m
        + config.w_risk * phi_r
        + config.w_budget * phi_b
}

/// Compute pressure for reduced state (T, E, Q).
/// Simplified pressure for minimal concrete instantiation.
pub fn compute_pressure_reduced(temperature: u128, energy: u128, queue: u128) -> u128 {
    // Simple linear pressure for reduced state
    // V = 2*T + E + Q (normalized)
    2 * temperature + energy + queue
}

/// Policy thresholds for Normal mode
pub struct NormalPolicy {
    pub t_max: u128,
    pub p_max: u128,
    pub e_min: u128,
    pub q_max: u128,
    pub delta_max: u128,
}

impl Default for NormalPolicy {
    fn default() -> Self {
        Self {
            t_max: 90000,    // 90°C
            p_max: 300000,   // 300W
            e_min: 10000,    // 10J
            q_max: 100,      // 100 queue depth
            delta_max: 5000, // 5 defect slack
        }
    }
}

/// Policy thresholds for Throttled mode
pub struct ThrottledPolicy {
    pub t_max: u128,
    pub p_max: u128,
    pub e_min: u128,
    pub q_max: u128,
    pub delta_max: u128,
}

impl Default for ThrottledPolicy {
    fn default() -> Self {
        Self {
            t_max: 75000,    // 75°C
            p_max: 220000,   // 220W
            e_min: 25000,    // 25J
            q_max: 80,       // 80 queue depth
            delta_max: 3000, // 3 defect slack
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gccp::state::{GccpState, PowerState, QueueState, ThermalState};

    #[test]
    fn test_thermal_pressure() {
        let config = PressureConfig::default();
        let state = ThermalState::new(60000, 70000, 1000);

        // Below cap (60°C)
        let p = thermal_pressure(&state, &config, 80000);
        assert_eq!(p, config.lambda1 * 60000 + config.lambda2 * 70000);

        // Above cap (80°C)
        let state_hot = ThermalState::new(90000, 90000, 5000);
        let p_hot = thermal_pressure(&state_hot, &config, 80000);
        // Should include lambda3*(90000-80000)
        assert!(p_hot > p);
    }

    #[test]
    fn test_power_pressure() {
        let config = PressureConfig::default();
        let state = PowerState::new(150000, 300000, 150000);

        let p = power_pressure(&state, &config);
        assert_eq!(p, config.mu1 * 150000); // Below cap, no overage
    }

    #[test]
    fn test_reduced_pressure() {
        let p = compute_pressure_reduced(50000, 100, 10);
        // 2*50000 + 100 + 10 = 100110
        assert_eq!(p, 100110);
    }

    #[test]
    fn test_normal_vs_throttled() {
        let normal = NormalPolicy::default();
        let throttled = ThrottledPolicy::default();

        // Normal allows higher temp
        assert!(normal.t_max > throttled.t_max);
        // Throttled requires more energy reserve
        assert!(throttled.e_min > normal.e_min);
    }
}
