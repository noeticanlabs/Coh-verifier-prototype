use crate::types::{Decision, MicroReceiptWire};
use crate::verify_micro;
use crate::trajectory::types::{
    AdmissibleTrajectory, CandidateEdge, DomainState, Action, VerifiedStep, AcceptWitness, witness_vector
};
use crate::trajectory::domain::{admissible_actions, derive_state};
use crate::trajectory::scoring::{calculate_score, ScoringWeights};
use crate::trajectory::search_result::SearchResult;

pub struct SearchContext {
    pub initial_state: DomainState,
    pub target_state: DomainState,
    pub max_depth: usize,
    pub beam_width: usize,
    pub weights: ScoringWeights,
}

/// The core Trajectory Engine implementing the 6-step pipeline
pub fn search(ctx: &SearchContext) -> SearchResult {
    let mut result = SearchResult::new();
    let mut frontier = vec![AdmissibleTrajectory::new()];
    
    // Initial state setup (In real impl, we might need a starting step)
    // For now, trajectories start empty and expand from ctx.initial_state
    
    for depth in 0..ctx.max_depth {
        let mut next_frontier = Vec::new();
        result.frontier_stats.max_depth_reached = depth + 1;

        for traj in frontier {
            let current_semantic_state = traj.steps.last()
                .map(|s| &s.state_next)
                .unwrap_or(&ctx.initial_state);

            // Step 1: Expand
            let actions = admissible_actions(current_semantic_state);
            
            for action in actions {
                result.frontier_stats.total_expanded += 1;

                // Step 2: Construct (and Derive state)
                let next_semantic_state = derive_state(current_semantic_state, &action);
                let wire = mock_receipt_wire(current_semantic_state, &action, &next_semantic_state);

                // Step 3: Verify
                let verification = verify_micro(wire.clone());

                // Step 4: Filter & Map Witness
                let witnesses = witness_vector(&verification);
                let is_accepted = verification.decision == Decision::Accept;

                if is_accepted {
                    // Step 5: Extend (Requires AcceptWitness)
                    let step = VerifiedStep {
                        state_prev: current_semantic_state.clone(),
                        action: action.clone(),
                        state_next: next_semantic_state.clone(),
                        witness: AcceptWitness, // Type-enforced admissibility
                    };
                    
                    let mut next_traj = traj.clone();
                    next_traj.push(step);
                    
                    // Step 6: Score Admissible Only
                    next_traj.cumulative_score = calculate_score(&next_traj, &ctx.target_state, &ctx.weights);
                    
                    next_frontier.push(next_traj);
                    result.frontier_stats.admissible_found += 1;
                } else {
                    // Capture for Rejected graph
                    result.rejected.push(CandidateEdge {
                        state_prev: current_semantic_state.clone(),
                        action: action.clone(),
                        state_next: next_semantic_state.clone(),
                        receipt: wire,
                        verification,
                        witnesses,
                    });
                    result.frontier_stats.rejected_found += 1;
                }
            }
        }

        // Beam pruning
        next_frontier.sort_by(|a, b| b.cumulative_score.partial_cmp(&a.cumulative_score).unwrap());
        frontier = next_frontier.into_iter().take(ctx.beam_width).collect();
        
        if frontier.is_empty() {
            break;
        }
    }

    result.admissible = frontier;
    result
}

/// Mock helper to create a wire receipt for a semantic transition
fn mock_receipt_wire(prev: &DomainState, action: &Action, next: &DomainState) -> MicroReceiptWire {
    let (v_pre, v_post) = match (prev, next) {
        (DomainState::Financial(f1), DomainState::Financial(f2)) => (f1.balance, f2.balance),
        _ => (100, 100),
    };

    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "traj.edge".to_string(),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09".to_string(),
        policy_hash: "0".repeat(64),
        step_index: 0,
        step_type: Some(format!("{:?}", action)),
        signatures: Some(vec![crate::types::SignatureWire {
            signature: "sig".to_string(),
            signer: "system".to_string(),
            timestamp: 0,
        }]),
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: crate::types::MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: (v_pre.saturating_sub(v_post)).to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    // Compute valid digest to satisfy C5
    if let Ok(r) = crate::types::MicroReceipt::try_from(wire.clone()) {
        let prehash = crate::canon::to_prehash_view(&r);
        if let Ok(bytes) = crate::canon::to_canonical_json_bytes(&prehash) {
            wire.chain_digest_next = crate::hash::compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();
        }
    }

    wire
}
