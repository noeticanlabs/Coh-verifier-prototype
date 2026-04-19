use serde::{Deserialize, Serialize};

// ============================================================================
// FINANCIAL DOMAIN: Canonical State Machine
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FinancialStatus {
    Idle,
    Invoiced,
    ReadyToPay,
    Paid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinancialState {
    pub balance: u128,
    pub initial_balance: u128,
    pub status: FinancialStatus,
    pub current_invoice_amount: u128,
}

impl FinancialState {
    pub fn safety_margin(&self) -> f64 {
        if self.initial_balance == 0 { return 1.0; }
        (self.balance as f64 / self.initial_balance as f64).clamp(0.0, 1.0)
    }

    pub fn progress_index(&self) -> f64 {
        match self.status {
            FinancialStatus::Idle => 0.0,
            FinancialStatus::Invoiced => 0.3,
            FinancialStatus::ReadyToPay => 0.7,
            FinancialStatus::Paid => 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FinancialAction {
    CreateInvoice { amount: u128 },
    VerifyVendor,
    IssuePayment { amount: u128 },
}

// ============================================================================
// AGENT DOMAIN: Canonical State Machine
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Observing,
    Acting,
    PolicyReview,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentState {
    pub complexity_index: u64,
    pub complexity_budget: u64,
    pub authority_level: u8,
    pub status: AgentStatus,
}

impl AgentState {
    pub fn safety_margin(&self) -> f64 {
        if self.complexity_budget == 0 { return 1.0; }
        (1.0 - (self.complexity_index as f64 / self.complexity_budget as f64)).clamp(0.0, 1.0)
    }

    pub fn progress_index(&self) -> f64 {
        match self.status {
            AgentStatus::Observing => 0.1,
            AgentStatus::Acting => 0.5,
            AgentStatus::PolicyReview => 0.2, // Setback for safety check
            AgentStatus::Completed => 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentAction {
    RetrieveData,
    CallTool { tool_id: String },
    UpdatePolicy,
    Finalize,
}

// ============================================================================
// OPS DOMAIN: Canonical State Machine
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpsStatus {
    Open,
    InProgress,
    MaterialsLogged,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpsState {
    pub status: OpsStatus,
    pub materials_logged: bool,
    pub stall_risk: f64, // Graded safety margin [0, 1]
    pub resource_readiness: f64, // Graded readiness [0, 1]
}

impl OpsState {
    pub fn safety_margin(&self) -> f64 {
        // Compositional margin: min of stall risk and resource readiness
        (1.0 - self.stall_risk).min(self.resource_readiness).clamp(0.0, 1.0)
    }

    pub fn progress_index(&self) -> f64 {
        match self.status {
            OpsStatus::Open => 0.0,
            OpsStatus::InProgress => 0.3,
            OpsStatus::MaterialsLogged => 0.7,
            OpsStatus::Closed => 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpsAction {
    OpenWorkOrder,
    StartWork,
    LogMaterials,
    CloseTicket,
}

use crate::trajectory::types::{DomainState, Action};

/// Get admissible actions based on current semantic state
pub fn admissible_actions(state: &DomainState) -> Vec<Action> {
    match state {
        DomainState::Financial(fs) => {
            match fs.status {
                FinancialStatus::Idle => vec![Action::Financial(FinancialAction::CreateInvoice { amount: 1000 })],
                FinancialStatus::Invoiced => vec![Action::Financial(FinancialAction::VerifyVendor)],
                FinancialStatus::ReadyToPay => {
                    if fs.balance >= fs.current_invoice_amount {
                        vec![Action::Financial(FinancialAction::IssuePayment { amount: fs.current_invoice_amount })]
                    } else {
                        vec![] 
                    }
                }
                FinancialStatus::Paid => vec![],
            }
        }
        DomainState::Agent(as_state) => {
            match as_state.status {
                AgentStatus::Observing => vec![Action::Agent(AgentAction::RetrieveData)],
                AgentStatus::Acting => vec![
                    Action::Agent(AgentAction::CallTool { tool_id: "search".to_string() }),
                    Action::Agent(AgentAction::UpdatePolicy),
                    Action::Agent(AgentAction::Finalize),
                ],
                AgentStatus::PolicyReview => vec![Action::Agent(AgentAction::RetrieveData)],
                AgentStatus::Completed => vec![], 
            }
        }
        DomainState::Ops(os) => {
            match os.status {
                OpsStatus::Open => vec![Action::Ops(OpsAction::StartWork)],
                OpsStatus::InProgress => vec![Action::Ops(OpsAction::LogMaterials)],
                OpsStatus::MaterialsLogged => vec![Action::Ops(OpsAction::CloseTicket)],
                OpsStatus::Closed => vec![], 
            }
        }
    }
}

/// Derive next semantic state
pub fn derive_state(state: &DomainState, action: &Action) -> DomainState {
    match (state, action) {
        (DomainState::Financial(fs), Action::Financial(fa)) => {
            let mut next = fs.clone();
            match fa {
                FinancialAction::CreateInvoice { amount } => {
                    if fs.status == FinancialStatus::Idle {
                        next.status = FinancialStatus::Invoiced;
                        next.current_invoice_amount = *amount;
                    }
                }
                FinancialAction::VerifyVendor => {
                    if fs.status == FinancialStatus::Invoiced {
                        next.status = FinancialStatus::ReadyToPay;
                    }
                }
                FinancialAction::IssuePayment { amount } => {
                    if fs.status == FinancialStatus::ReadyToPay && fs.balance >= *amount {
                        next.status = FinancialStatus::Paid;
                        next.balance = next.balance.saturating_sub(*amount);
                    }
                }
            }
            DomainState::Financial(next)
        }
        (DomainState::Agent(as_state), Action::Agent(aa)) => {
            let mut next = as_state.clone();
            match aa {
                AgentAction::RetrieveData => {
                    next.complexity_index += 1;
                    next.status = if as_state.status == AgentStatus::PolicyReview {
                        AgentStatus::Observing
                    } else {
                        AgentStatus::Acting
                    };
                }
                AgentAction::CallTool { .. } => {
                    next.complexity_index += 2;
                }
                AgentAction::UpdatePolicy => {
                    next.authority_level += 1;
                    next.status = AgentStatus::PolicyReview;
                }
                AgentAction::Finalize => {
                    next.status = AgentStatus::Completed;
                }
            }
            DomainState::Agent(next)
        }
        (DomainState::Ops(os), Action::Ops(oa)) => {
            let mut next = os.clone();
            match oa {
                OpsAction::OpenWorkOrder => next.status = OpsStatus::Open,
                OpsAction::StartWork => next.status = OpsStatus::InProgress,
                OpsAction::LogMaterials => {
                    next.materials_logged = true;
                    next.status = OpsStatus::MaterialsLogged;
                }
                OpsAction::CloseTicket => next.status = OpsStatus::Closed,
            }
            DomainState::Ops(next)
        }
        _ => state.clone(), 
    }
}

impl DomainState {
    pub fn safety_margin(&self) -> f64 {
        match self {
            DomainState::Financial(fs) => fs.safety_margin(),
            DomainState::Agent(as_state) => as_state.safety_margin(),
            DomainState::Ops(os) => os.safety_margin(),
        }
    }

    pub fn progress_index(&self) -> f64 {
        match self {
            DomainState::Financial(fs) => fs.progress_index(),
            DomainState::Agent(as_state) => as_state.progress_index(),
            DomainState::Ops(os) => os.progress_index(),
        }
    }
}
