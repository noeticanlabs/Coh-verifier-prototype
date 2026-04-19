//! GCCP State Types
//!
//! Implements the GCCP v1 state space as specified in the Governed Compute Control Plane.
//! State is represented as an 8-tuple: (T, P, Q, U, M, R, B, Πc)
//!
//! - T: Thermal state (die temp, hotspot temp, thermal slope)
//! - P: Power state (instantaneous draw, cap, margin)
//! - Q: Queue state (depth, age pressure, class mix)
//! - U: Utilization state (compute, memory bandwidth, interconnect)
//! - M: Memory state (used, bandwidth, fragmentation)
//! - R: Risk/Instability state (retry, timeout, throttling)
//! - B: Budgets (energy, latency, stability)
//! - Πc: Control context (policy hash, profile hash, mode, workload class)

use serde::{Deserialize, Serialize};

/// Thermal state: die temperature, hotspot temperature, and recent thermal slope.
/// All values are in fixed-point representation (scaled by 1000 for sub-degree precision).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThermalState {
    /// Die temperature in millidegrees Celsius (e.g., 85000 = 85.0°C)
    pub t_die: u128,
    /// Hotspot temperature in millidegrees Celsius
    pub t_hot: u128,
    /// Recent thermal slope in millidegrees per second (positive = rising)
    pub t_rise: i128,
}

impl ThermalState {
    pub fn new(t_die: u128, t_hot: u128, t_rise: i128) -> Self {
        Self {
            t_die,
            t_hot,
            t_rise,
        }
    }

    pub fn default() -> Self {
        Self {
            t_die: 45000, // 45°C default
            t_hot: 50000, // 50°C default
            t_rise: 0,    // neutral
        }
    }
}

/// Power state: instantaneous draw, cap, and remaining margin.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerState {
    /// Instantaneous power draw in milliwatts (e.g., 150000 = 150W)
    pub p_now: u128,
    /// Power cap in milliwatts
    pub p_cap: u128,
    /// Remaining margin in milliwatts (p_cap - p_now)
    pub p_margin: u128,
}

impl PowerState {
    pub fn new(p_now: u128, p_cap: u128, p_margin: u128) -> Self {
        Self {
            p_now,
            p_cap,
            p_margin,
        }
    }

    pub fn default() -> Self {
        Self {
            p_now: 50000,  // 50W default
            p_cap: 300000, // 300W cap
            p_margin: 250000,
        }
    }
}

/// Queue state: queue depth, age pressure, and class mix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueState {
    /// Current queue depth (number of pending items)
    pub q_depth: u128,
    /// Age pressure (cumulative wait time in milliseconds)
    pub q_age: u128,
    /// Class mix - bitmask representing workload class distribution
    /// Bit 0 = compute-heavy, Bit 1 = memory-heavy, Bit 2 = I/O-bound
    pub q_mix: u128,
}

impl QueueState {
    pub fn new(q_depth: u128, q_age: u128, q_mix: u128) -> Self {
        Self {
            q_depth,
            q_age,
            q_mix,
        }
    }

    pub fn default() -> Self {
        Self {
            q_depth: 0,
            q_age: 0,
            q_mix: 0,
        }
    }
}

/// Utilization state: compute occupancy, memory bandwidth load, interconnect pressure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtilizationState {
    /// Compute core occupancy (0-1000 = 0-100%)
    pub u_core: u128,
    /// Memory bandwidth load (0-1000 = 0-100%)
    pub u_mem: u128,
    /// Interconnect pressure (0-1000 = 0-100%)
    pub u_link: u128,
}

impl UtilizationState {
    pub fn new(u_core: u128, u_mem: u128, u_link: u128) -> Self {
        Self {
            u_core,
            u_mem,
            u_link,
        }
    }

    pub fn default() -> Self {
        Self {
            u_core: 0,
            u_mem: 0,
            u_link: 0,
        }
    }
}

/// Memory state: used memory, bandwidth load, fragmentation/coherence pressure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryState {
    /// Used memory in megabytes
    pub m_used: u128,
    /// Memory bandwidth load (0-1000 = 0-100%)
    pub m_bw: u128,
    /// Fragmentation/coherence pressure (0-1000 = 0-100%)
    pub m_frag: u128,
}

impl MemoryState {
    pub fn new(m_used: u128, m_bw: u128, m_frag: u128) -> Self {
        Self {
            m_used,
            m_bw,
            m_frag,
        }
    }

    pub fn default() -> Self {
        Self {
            m_used: 0,
            m_bw: 0,
            m_frag: 0,
        }
    }
}

/// Risk/Instability state: retry pressure, timeout pressure, and throttling pressure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskState {
    /// Retry pressure (cumulative retry count)
    pub r_retry: u128,
    /// Timeout pressure (cumulative timeout count)
    pub r_timeout: u128,
    /// Throttling pressure (cumulative throttle events)
    pub r_throttle: u128,
}

impl RiskState {
    pub fn new(r_retry: u128, r_timeout: u128, r_throttle: u128) -> Self {
        Self {
            r_retry,
            r_timeout,
            r_throttle,
        }
    }

    pub fn default() -> Self {
        Self {
            r_retry: 0,
            r_timeout: 0,
            r_throttle: 0,
        }
    }
}

/// Budget state: energy, latency, and stability budgets.
/// All values are in their respective units (joules, milliseconds, count).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetState {
    /// Energy budget in joules remaining
    pub b_energy: u128,
    /// Latency budget in milliseconds remaining
    pub b_latency: u128,
    /// Stability budget (risk events allowed)
    pub b_stability: u128,
}

impl BudgetState {
    pub fn new(b_energy: u128, b_latency: u128, b_stability: u128) -> Self {
        Self {
            b_energy,
            b_latency,
            b_stability,
        }
    }

    pub fn default() -> Self {
        Self {
            b_energy: 1000,   // 1kJ default
            b_latency: 10000, // 10s default
            b_stability: 100, // 100 events default
        }
    }
}

/// Operating mode for GCCP control plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    /// Full speed, normal thermal/power envelopes
    Normal,
    /// Reduced thermal/power limits
    Throttled,
    /// Conservative mode for critical workloads
    Safe,
    /// Minimal mode for battery/thermal emergency
    Minimal,
}

impl Default for OperatingMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Workload class for classification and routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadClass {
    /// Compute-intensive (CUDA kernels, ML training)
    Compute,
    /// Memory-intensive (database, caching)
    Memory,
    /// I/O-intensive (networking, storage)
    Io,
    /// Mixed or unknown
    Mixed,
}

impl Default for WorkloadClass {
    fn default() -> Self {
        Self::Mixed
    }
}

/// Control context: policy hash, canon profile hash, operating mode, workload class.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlContext {
    /// Policy hash (SHA-256 hex)
    pub policy_hash: String,
    /// Canon profile hash (SHA-256 hex)
    pub profile_hash: String,
    /// Operating mode
    pub mode: OperatingMode,
    /// Workload class
    pub class: WorkloadClass,
}

impl ControlContext {
    pub fn new(
        policy_hash: String,
        profile_hash: String,
        mode: OperatingMode,
        class: WorkloadClass,
    ) -> Self {
        Self {
            policy_hash,
            profile_hash,
            mode,
            class,
        }
    }

    pub fn default() -> Self {
        Self {
            policy_hash: "default_policy".to_string(),
            profile_hash: "default_profile".to_string(),
            mode: OperatingMode::Normal,
            class: WorkloadClass::Mixed,
        }
    }
}

/// Full GCCP state: 8-tuple (T, P, Q, U, M, R, B, Πc)
/// This is the complete state representation for the Governed Compute Control Plane.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GccpState {
    /// Thermal state
    pub thermal: ThermalState,
    /// Power state
    pub power: PowerState,
    /// Queue state
    pub queue: QueueState,
    /// Utilization state
    pub utilization: UtilizationState,
    /// Memory state
    pub memory: MemoryState,
    /// Risk/instability state
    pub risk: RiskState,
    /// Budget state
    pub budgets: BudgetState,
    /// Control context
    pub context: ControlContext,
}

impl GccpState {
    pub fn new(
        thermal: ThermalState,
        power: PowerState,
        queue: QueueState,
        utilization: UtilizationState,
        memory: MemoryState,
        risk: RiskState,
        budgets: BudgetState,
        context: ControlContext,
    ) -> Self {
        Self {
            thermal,
            power,
            queue,
            utilization,
            memory,
            risk,
            budgets,
            context,
        }
    }

    pub fn default() -> Self {
        Self {
            thermal: ThermalState::default(),
            power: PowerState::default(),
            queue: QueueState::default(),
            utilization: UtilizationState::default(),
            memory: MemoryState::default(),
            risk: RiskState::default(),
            budgets: BudgetState::default(),
            context: ControlContext::default(),
        }
    }
}

/// Reduced GCCP state for minimal concrete instantiation (Section 31).
/// Uses only (T, E, Q) where E is energy instead of full power state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GccpStateReduced {
    /// Thermal state (die temperature in °C * 1000)
    pub temperature: u128,
    /// Energy consumed in joules
    pub energy: u128,
    /// Queue depth
    pub queue: u128,
}

impl GccpStateReduced {
    pub fn new(temperature: u128, energy: u128, queue: u128) -> Self {
        Self {
            temperature,
            energy,
            queue,
        }
    }

    pub fn default() -> Self {
        Self {
            temperature: 45000, // 45°C
            energy: 0,
            queue: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_state_default() {
        let t = ThermalState::default();
        assert_eq!(t.t_die, 45000);
        assert_eq!(t.t_hot, 50000);
        assert_eq!(t.t_rise, 0);
    }

    #[test]
    fn test_gccp_state_default() {
        let state = GccpState::default();
        assert_eq!(state.thermal.t_die, 45000);
        assert_eq!(state.power.p_cap, 300000);
        assert_eq!(state.context.mode, OperatingMode::Normal);
    }

    #[test]
    fn test_reduced_state() {
        let state = GccpStateReduced::new(85000, 100, 50);
        assert_eq!(state.temperature, 85000);
        assert_eq!(state.energy, 100);
        assert_eq!(state.queue, 50);
    }
}
