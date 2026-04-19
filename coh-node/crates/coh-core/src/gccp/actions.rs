//! GCCP Action Types
//!
//! Implements the GCCP v1 action space as specified in the Governed Compute Control Plane.
//! Actions are typed variants: dispatch, defer, reroute, resize, cooldown, reject

use serde::{Deserialize, Serialize};

use crate::reject::RejectCode;

/// Dispatch action: execute kernel on device with specified parameters.
/// Corresponds to GCCP action adispatch = (k, d, s, β, π)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchAction {
    /// Kernel identifier (shader name, function name, etc.)
    pub kernel: String,
    /// Target device (GPU0, GPU1, CPU, etc.)
    pub device: String,
    /// Stream identifier for ordering
    pub stream: String,
    /// Batch size (number of items to process)
    pub batch: u128,
    /// Priority (0-255, higher = more urgent)
    pub priority: u8,
}

impl DispatchAction {
    pub fn new(kernel: String, device: String, stream: String, batch: u128, priority: u8) -> Self {
        Self {
            kernel,
            device,
            stream,
            batch,
            priority,
        }
    }
}

/// Defer action: hold kernel for specified interval before execution.
/// Corresponds to GCCP action adefer = (k, Δt)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferAction {
    /// Kernel identifier to defer
    pub kernel: String,
    /// Deferral interval in milliseconds
    pub delta_t: u128,
}

impl DeferAction {
    pub fn new(kernel: String, delta_t: u128) -> Self {
        Self { kernel, delta_t }
    }
}

/// Reroute action: move kernel to alternate device.
/// Corresponds to GCCP action areroute = (k, d')
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RerouteAction {
    /// Kernel identifier to reroute
    pub kernel: String,
    /// Alternate target device
    pub target: String,
}

impl RerouteAction {
    pub fn new(kernel: String, target: String) -> Self {
        Self { kernel, target }
    }
}

/// Resize action: shrink or expand batch size.
/// Corresponds to GCCP action aresize = (k, β')
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResizeAction {
    /// Kernel identifier to resize
    pub kernel: String,
    /// New batch size
    pub new_batch: u128,
}

impl ResizeAction {
    pub fn new(kernel: String, new_batch: u128) -> Self {
        Self { kernel, new_batch }
    }
}

/// Cooldown action: reserve thermal recovery window.
/// Corresponds to GCCP action acooldown = (d, Δt)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CooldownAction {
    /// Device to cooldown
    pub device: String,
    /// Recovery window in milliseconds
    pub delta_t: u128,
}

impl CooldownAction {
    pub fn new(device: String, delta_t: u128) -> Self {
        Self { device, delta_t }
    }
}

/// Reject action: drop the action with specified reject code.
/// Corresponds to GCCP action areject = (k, c)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectAction {
    /// Kernel identifier that was rejected
    pub kernel: String,
    /// Reject code explaining why
    pub code: RejectCode,
}

impl RejectAction {
    pub fn new(kernel: String, code: RejectCode) -> Self {
        Self { kernel, code }
    }
}

/// GCCP Action enum: full action space
/// Variants: dispatch, defer, reroute, resize, cooldown, reject
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action_type", content = "params")]
pub enum GccpAction {
    /// Dispatch kernel to device
    Dispatch(DispatchAction),
    /// Defer execution for interval
    Defer(DeferAction),
    /// Reroute to alternate device
    Reroute(RerouteAction),
    /// Resize batch
    Resize(ResizeAction),
    /// Cooldown thermal recovery
    Cooldown(CooldownAction),
    /// Explicit rejection
    Reject(RejectAction),
}

impl GccpAction {
    /// Get the kernel identifier if this action has one
    pub fn kernel(&self) -> Option<&str> {
        match self {
            GccpAction::Dispatch(a) => Some(&a.kernel),
            GccpAction::Defer(a) => Some(&a.kernel),
            GccpAction::Reroute(a) => Some(&a.kernel),
            GccpAction::Resize(a) => Some(&a.kernel),
            GccpAction::Cooldown(_) => None,
            GccpAction::Reject(a) => Some(&a.kernel),
        }
    }

    /// Get the target device if this action references one
    pub fn target_device(&self) -> Option<&str> {
        match self {
            GccpAction::Dispatch(a) => Some(&a.device),
            GccpAction::Reroute(a) => Some(&a.target),
            GccpAction::Cooldown(a) => Some(&a.device),
            _ => None,
        }
    }
}

/// Discrete action levels for minimal concrete instantiation (Section 31).
/// Simplified action set for proof-of-concept.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GccpActionDiscrete {
    /// Idle - no operation
    Idle,
    /// Light workload
    Light,
    /// Medium workload
    Medium,
    /// Heavy workload
    Heavy,
    /// Cooldown/thermal recovery
    Cool,
}

impl GccpActionDiscrete {
    /// Get the expected energy cost for this action level
    pub fn energy_cost(&self) -> u128 {
        match self {
            GccpActionDiscrete::Idle => 0,
            GccpActionDiscrete::Light => 10,
            GccpActionDiscrete::Medium => 50,
            GccpActionDiscrete::Heavy => 150,
            GccpActionDiscrete::Cool => 5,
        }
    }

    /// Get the expected thermal rise for this action level (millidegrees C)
    pub fn thermal_rise(&self) -> u128 {
        match self {
            GccpActionDiscrete::Idle => 0,
            GccpActionDiscrete::Light => 1000,
            GccpActionDiscrete::Medium => 5000,
            GccpActionDiscrete::Heavy => 20000,
            GccpActionDiscrete::Cool => 0,
        }
    }

    /// Get the expected queue impact for this action level
    pub fn queue_impact(&self) -> i128 {
        match self {
            GccpActionDiscrete::Idle => 0,
            GccpActionDiscrete::Light => -1,
            GccpActionDiscrete::Medium => 0,
            GccpActionDiscrete::Heavy => 10,
            GccpActionDiscrete::Cool => 0,
        }
    }
}

impl Default for GccpActionDiscrete {
    fn default() -> Self {
        Self::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_action() {
        let action = GccpAction::Dispatch(DispatchAction::new(
            "matmul_kernel".to_string(),
            "GPU0".to_string(),
            "stream0".to_string(),
            1024,
            128,
        ));
        assert_eq!(action.kernel(), Some("matmul_kernel"));
        assert_eq!(action.target_device(), Some("GPU0"));
    }

    #[test]
    fn test_discrete_action_costs() {
        assert_eq!(GccpActionDiscrete::Idle.energy_cost(), 0);
        assert_eq!(GccpActionDiscrete::Heavy.energy_cost(), 150);
        assert_eq!(GccpActionDiscrete::Heavy.thermal_rise(), 20000);
    }

    #[test]
    fn test_discrete_action_queue_impact() {
        assert_eq!(GccpActionDiscrete::Light.queue_impact(), -1);
        assert_eq!(GccpActionDiscrete::Heavy.queue_impact(), 10);
    }
}
