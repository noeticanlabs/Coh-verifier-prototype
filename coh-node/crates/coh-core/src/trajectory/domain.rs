use serde::{Deserialize, Serialize};

// ============================================================================
// FINANCIAL DOMAIN
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinancialState {
    pub balance: u128,
    pub vendor_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FinancialAction {
    CreateInvoice { amount: u128 },
    VerifyVendor,
    IssuePayment { amount: u128 },
}

// ============================================================================
// AGENT DOMAIN
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentState {
    pub complexity_index: u64,
    pub authority_level: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentAction {
    RetrieveData,
    CallTool { tool_id: String },
    UpdatePolicy,
}

// ============================================================================
// OPS DOMAIN
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpsState {
    pub status: String, // "Open", "InProgress", "Closed"
    pub materials_logged: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpsAction {
    OpenWorkOrder,
    LogMaterials,
    CloseTicket,
}

use crate::trajectory::types::{DomainState, Action};

/// Get admissible actions based on current semantic state
pub fn admissible_actions(state: &DomainState) -> Vec<Action> {
    match state {
        DomainState::Financial(fs) => {
            let mut actions = vec![Action::Financial(FinancialAction::CreateInvoice { amount: 1000 })];
            if !fs.vendor_verified {
                actions.push(Action::Financial(FinancialAction::VerifyVendor));
            }
            if fs.vendor_verified && fs.balance >= 1000 {
                actions.push(Action::Financial(FinancialAction::IssuePayment { amount: 1000 }));
            }
            actions
        }
        DomainState::Agent(_as) => {
            vec![
                Action::Agent(AgentAction::RetrieveData),
                Action::Agent(AgentAction::CallTool { tool_id: "search".to_string() }),
            ]
        }
        DomainState::Ops(os) => {
            let mut actions = Vec::new();
            if os.status == "Open" {
                actions.push(Action::Ops(OpsAction::LogMaterials));
            }
            if os.materials_logged && os.status != "Closed" {
                actions.push(Action::Ops(OpsAction::CloseTicket));
            }
            actions
        }
    }
}

/// Derive next semantic state (Canonical transition)
pub fn derive_state(state: &DomainState, action: &Action) -> DomainState {
    match (state, action) {
        (DomainState::Financial(fs), Action::Financial(fa)) => {
            let mut next = fs.clone();
            match fa {
                FinancialAction::CreateInvoice { .. } => {}
                FinancialAction::VerifyVendor => next.vendor_verified = true,
                FinancialAction::IssuePayment { amount } => next.balance = next.balance.saturating_sub(*amount),
            }
            DomainState::Financial(next)
        }
        (DomainState::Agent(as_state), Action::Agent(aa)) => {
            let mut next = as_state.clone();
            match aa {
                AgentAction::RetrieveData => next.complexity_index += 1,
                AgentAction::CallTool { .. } => next.complexity_index += 2,
                AgentAction::UpdatePolicy => next.authority_level += 1,
            }
            DomainState::Agent(next)
        }
        (DomainState::Ops(os), Action::Ops(oa)) => {
            let mut next = os.clone();
            match oa {
                OpsAction::OpenWorkOrder => next.status = "Open".to_string(),
                OpsAction::LogMaterials => next.materials_logged = true,
                OpsAction::CloseTicket => next.status = "Closed".to_string(),
            }
            DomainState::Ops(next)
        }
        _ => state.clone(), // Invalid mixed domain transitions
    }
}

/// Goal distance heuristic
pub fn goal_distance(state: &DomainState, target: &DomainState) -> f64 {
    match (state, target) {
        (DomainState::Financial(fs), DomainState::Financial(ts)) => {
            if fs.balance == ts.balance { 0.0 } else { 1.0 }
        }
        (DomainState::Ops(os), DomainState::Ops(ts)) => {
            if os.status == ts.status { 0.0 } else { 1.0 }
        }
        _ => 1.0,
    }
}
