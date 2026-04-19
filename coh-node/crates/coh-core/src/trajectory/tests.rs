#[cfg(test)]
mod tests {
    use crate::trajectory::types::{DomainState, AcceptWitness, VerifiedStep, AdmissibleTrajectory, Action};
    use crate::trajectory::domain::{
        FinancialState, FinancialStatus, FinancialAction, 
        OpsState, OpsStatus, AgentState, AgentStatus
    };
    use crate::trajectory::engine::{search, SearchContext};
    use crate::trajectory::scoring::{evaluate_path, ScoringWeights};
    use crate::types::Hash32;

    #[test]
    fn test_lexicographic_safety_priority() {
        // Path A: Lower progress, Higher safety
        let mut traj_a = AdmissibleTrajectory::new();
        traj_a.steps.push(VerifiedStep::new(
            DomainState::Financial(FinancialState { balance: 10000, initial_balance: 10000, status: FinancialStatus::Idle, current_invoice_amount: 0 }),
            Action::Financial(FinancialAction::CreateInvoice { amount: 1000 }),
            DomainState::Financial(FinancialState { balance: 10000, initial_balance: 10000, status: FinancialStatus::Invoiced, current_invoice_amount: 1000 }),
            Hash32::default(), Hash32::default(), AcceptWitness
        ));
        let eval_a = evaluate_path(&traj_a, 10); // Safety: 1.0, Progress: 0.3

        // Path B: Higher progress, Lower safety
        let mut traj_b = AdmissibleTrajectory::new();
        traj_b.steps.push(VerifiedStep::new(
            DomainState::Financial(FinancialState { balance: 5000, initial_balance: 10000, status: FinancialStatus::Idle, current_invoice_amount: 0 }),
            Action::Financial(FinancialAction::CreateInvoice { amount: 1000 }),
            DomainState::Financial(FinancialState { balance: 5000, initial_balance: 10000, status: FinancialStatus::Invoiced, current_invoice_amount: 1000 }),
            Hash32::default(), Hash32::default(), AcceptWitness
        ));
        traj_b.steps.push(VerifiedStep::new(
            DomainState::Financial(FinancialState { balance: 5000, initial_balance: 10000, status: FinancialStatus::Invoiced, current_invoice_amount: 1000 }),
            Action::Financial(FinancialAction::VerifyVendor),
            DomainState::Financial(FinancialState { balance: 5000, initial_balance: 10000, status: FinancialStatus::ReadyToPay, current_invoice_amount: 1000 }),
            Hash32::default(), Hash32::default(), AcceptWitness
        ));
        let eval_b = evaluate_path(&traj_b, 10); // Safety: 0.5, Progress: 0.7

        // Safety Bottleneck Rule: Path A (1.0 safety) MUST beat Path B (0.5 safety) despite inferior progress
        assert!(eval_a > eval_b);
    }

    #[test]
    fn test_ops_graded_safety() {
        let s_optimal = DomainState::Ops(OpsState { 
            status: OpsStatus::InProgress, 
            materials_logged: false, 
            stall_risk: 0.0, 
            resource_readiness: 1.0 
        });
        let s_risky = DomainState::Ops(OpsState { 
            status: OpsStatus::InProgress, 
            materials_logged: false, 
            stall_risk: 0.4, 
            resource_readiness: 1.0 
        });
        let s_blocked = DomainState::Ops(OpsState { 
            status: OpsStatus::InProgress, 
            materials_logged: false, 
            stall_risk: 0.0, 
            resource_readiness: 0.2 
        });

        assert_eq!(s_optimal.safety_margin(), 1.0);
        assert_eq!(s_risky.safety_margin(), 0.6);
        assert_eq!(s_blocked.safety_margin(), 0.2);
    }

    #[test]
    fn test_domain_progress_maps_bounded() {
        let f = DomainState::Financial(FinancialState { balance: 1000, initial_balance: 1000, status: FinancialStatus::Paid, current_invoice_amount: 1000 });
        let a = DomainState::Agent(AgentState { complexity_index: 0, complexity_budget: 100, authority_level: 0, status: AgentStatus::Completed });
        let o = DomainState::Ops(OpsState { status: OpsStatus::Closed, materials_logged: true, stall_risk: 0.0, resource_readiness: 1.0 });

        assert_eq!(f.progress_index(), 1.0);
        assert_eq!(a.progress_index(), 1.0);
        assert_eq!(o.progress_index(), 1.0);
    }

    #[test]
    fn test_engine_lexicographic_search() {
        let ctx = SearchContext {
            initial_state: DomainState::Financial(FinancialState { 
                balance: 5000, 
                initial_balance: 5000,
                status: FinancialStatus::Idle, 
                current_invoice_amount: 0 
            }),
            target_state: DomainState::Financial(FinancialState { 
                balance: 4000, 
                initial_balance: 5000,
                status: FinancialStatus::Paid, 
                current_invoice_amount: 1000 
            }),
            max_depth: 3,
            beam_width: 5,
            weights: ScoringWeights::default(),
        };

        let result = search(&ctx);
        
        assert!(result.frontier_stats.admissible_found > 0);
        // Best path should have evaluation stored
        let best = &result.admissible[0];
        assert!(best.evaluation.is_some());
    }
}
