#[cfg(test)]
mod tests {
    use crate::trajectory::types::{DomainState, AcceptWitness, VerifiedStep, AdmissibleTrajectory};
    use crate::trajectory::domain::{FinancialState, FinancialAction, OpsState, OpsAction};
    use crate::trajectory::engine::{search, SearchContext};
    use crate::trajectory::scoring::ScoringWeights;
    use crate::trajectory::types::Action;

    #[test]
    fn test_admissible_trajectory_invariant() {
        // We verify that we can create a valid step with the marker
        let step = VerifiedStep {
            state_prev: DomainState::Financial(FinancialState { balance: 1000, vendor_verified: false }),
            action: Action::Financial(FinancialAction::VerifyVendor),
            state_next: DomainState::Financial(FinancialState { balance: 1000, vendor_verified: true }),
            witness: AcceptWitness,
        };
        
        let mut traj = AdmissibleTrajectory::new();
        traj.push(step);
        assert_eq!(traj.steps.len(), 1);
    }

    #[test]
    fn test_domain_transition_ops() {
        let state = DomainState::Ops(OpsState { status: "Open".to_string(), materials_logged: false });
        let action = Action::Ops(OpsAction::LogMaterials);
        let next = crate::trajectory::domain::derive_state(&state, &action);
        
        if let DomainState::Ops(os) = next {
            assert!(os.materials_logged);
            assert_eq!(os.status, "Open");
        } else {
            panic!("Wrong domain");
        }
    }

    #[test]
    fn test_engine_search_admissible_only() {
        let ctx = SearchContext {
            initial_state: DomainState::Financial(FinancialState { balance: 5000, vendor_verified: false }),
            target_state: DomainState::Financial(FinancialState { balance: 3000, vendor_verified: true }),
            max_depth: 3,
            beam_width: 2,
            weights: ScoringWeights::default(),
        };

        let result = search(&ctx);
        
        // Assertions
        assert!(result.frontier_stats.admissible_found > 0);
        for traj in result.admissible {
            for step in traj.steps {
                // Every step in admissible set MUST have AcceptWitness (enforced by types)
                let _ = step.witness; 
            }
        }
    }
}
