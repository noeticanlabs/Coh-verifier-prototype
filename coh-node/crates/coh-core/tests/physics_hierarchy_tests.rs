use coh_core::cohbit::{CohBit, CohBitLaw, CohBitState};
use coh_core::atom::{CohAtom, AtomGeometry, AtomMetabolism};
use coh_physics::CohSpinor;
use coh_physics::current::CoherenceCurrent;
use coh_physics::gauge::YangMillsCurvature;
use coh_core::types::{Hash32, Decision};
use num_rational::Rational64;
use num_complex::Complex64;

#[test]
fn test_layer_1_cohbit_admissibility() {
    let bit = CohBit {
        from_state: Hash32([0; 32]),
        to_state: Hash32([1; 32]),
        transition_id: "test".to_string(),
        projection_hash: Hash32([2; 32]),
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(10, 1),
        defect: Rational64::new(5, 1),
        delta_hat: Rational64::new(5, 1),
        utility: 1.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([3; 32]),
        state: CohBitState::Superposed,
    };

    // m = V_pre + D - V_post - S = 100 + 5 - 90 - 10 = 5
    assert_eq!(bit.margin(), Rational64::new(5, 1));
    assert!(bit.is_executable());

    // Test soft probability normalization
    let mut bits = vec![bit.clone()];
    CohBitLaw::compute_soft_probabilities(&mut bits, 1.0, 1.0);
    assert!(bits[0].probability_soft > 0.99); // Only one bit
}

#[test]
fn test_layer_2_coh_atom_evolution() {
    let state_x = Hash32([0; 32]);
    let state_y = Hash32([1; 32]);
    
    let bit = CohBit {
        from_state: state_x,
        to_state: state_y,
        transition_id: "step_1".to_string(),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(5, 1),
        defect: Rational64::new(2, 1),
        delta_hat: Rational64::new(2, 1),
        utility: 10.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([2; 32]),
        state: CohBitState::Superposed,
    };

    let mut atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![bit.clone()],
        geometry: AtomGeometry {
            distance: Rational64::new(0, 1),
            curvature: 0.0,
            ricci_scalar: 0.1,
        },
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(10, 1),
        },
        receipt_chain: vec![],
    };

    // Evolve atom
    let success = atom.evolve(&bit, 1.0, 0.0);
    assert!(success);
    assert_eq!(atom.state_hash, state_y);
    assert_eq!(atom.valuation, Rational64::new(90, 1));
    // Budget: 1000 + 10 (refresh) - 5 (spend) = 1005
    assert_eq!(atom.metabolism.budget, Rational64::new(1005, 1));
}

#[test]
fn test_layer_3_coh_spinor_current() {
    // Construct a spinor corresponding to a Coh Atom with valuation 1.0
    let val: f64 = 1.0;
    let mut psi = CohSpinor::new(
        Complex64::new(val.sqrt(), 0.0), // psi_0 = sqrt(V)
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
    );
    
    // Verify J^0 = density = V
    assert!((psi.density() - val).abs() < 1e-10);
    
    let current = CoherenceCurrent::compute(&psi);
    assert!((current.j0 - val).abs() < 1e-10);
    assert_eq!(current.j1, 0.0);

    // Verify Effective Metric Coupling
    let g_base = [[1.0, 0.0, 0.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [0.0, 0.0, 0.0, -1.0]];
    let g_eff = current.effective_metric_coupling(g_base, 0.0, 0.1, 0.0);
    // g_eff[0][0] = 1.0 + 0.1 * J0 * J0 = 1.0 + 0.1 * 1.0 * 1.0 = 1.1
    assert!((g_eff[0][0] - 1.1).abs() < 1e-10);
}

#[test]
fn test_layer_7_wilson_loop_holonomy() {
    use coh_physics::gauge::{CohGaugeField, WilsonLoopReceipt};
    use coh_core::trajectory::path_integral::CohHistory;
    
    let mut gauge = CohGaugeField::new(3);
    gauge.connection[0][0] = 0.1; // Local verifier rotation
    
    let bit = CohBit {
        from_state: Hash32([0; 32]),
        to_state: Hash32([1; 32]),
        transition_id: "step".to_string(),
        projection_hash: Hash32([0; 32]),
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(100, 1),
        spend: Rational64::new(0, 1),
        defect: Rational64::new(0, 1),
        delta_hat: Rational64::new(0, 1),
        utility: 0.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([2; 32]),
        state: CohBitState::Superposed,
    };

    let history = CohHistory {
        steps: vec![bit.clone(); 10], // A loop of 10 steps
    };

    // Total phase = 10 * 0.1 = 1.0
    // W = (cos(1) + sin(1)) / 2 = (0.54 + 0.84) / 2 = 0.69
    let holonomy = WilsonLoopReceipt::compute_holonomy(&history, &gauge);
    assert!(holonomy > 0.6 && holonomy < 0.75);
    
    let receipt = WilsonLoopReceipt {
        path_hash: "test".to_string(),
        holonomy_trace: holonomy,
        curvature_sum: 0.0,
    };
    
    assert!(!receipt.is_admissible(0.1)); // 0.69 is not within 0.1 of 1.0
    assert!(receipt.is_admissible(0.4)); // 0.69 is within 0.4 of 1.0
}

#[test]
fn test_layer_6_path_integral_weighting() {
    use coh_core::trajectory::path_integral::{CohHistory, Propagator};
    use num_traits::ToPrimitive;
    
    let state_x = Hash32([0; 32]);
    let state_y = Hash32([1; 32]);
    let state_z = Hash32([2; 32]);
    
    let bit1 = CohBit {
        from_state: state_x,
        to_state: state_y,
        transition_id: "step1".to_string(),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(5, 1),
        defect: Rational64::new(0, 1),
        delta_hat: Rational64::new(2, 1),
        utility: 10.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([3; 32]),
        state: CohBitState::Superposed,
    };

    let bit2 = CohBit {
        from_state: state_y,
        to_state: state_z,
        transition_id: "step2".to_string(),
        projection_hash: state_y,
        valuation_pre: Rational64::new(90, 1),
        valuation_post: Rational64::new(80, 1),
        spend: Rational64::new(5, 1),
        defect: Rational64::new(0, 1),
        delta_hat: Rational64::new(2, 1),
        utility: 10.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([4; 32]),
        state: CohBitState::Superposed,
    };

    let history = CohHistory {
        steps: vec![bit1.clone(), bit2.clone()],
    };

    let atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![],
        geometry: AtomGeometry {
            distance: Rational64::new(0, 1),
            curvature: 0.0,
            ricci_scalar: 0.0,
        },
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(0, 1),
        },
        receipt_chain: vec![],
    };

    // Action J per bit = delta_hat - utility = 2 - 10 = -8
    // Total Action = -16
    // Path Probability P = e^{-(-16)/1.0} * sigmoid(beta * m1) * sigmoid(beta * m2)
    // m1 = 100 + 0 - 90 - 5 = 5
    // m2 = 90 + 0 - 80 - 5 = 5
    let prob = history.path_probability(&atom, 1.0, 0.0, 1.0, 1.0);
    assert!(prob > 0.0);
    
    let z = Propagator::partition_function(&[history], &atom, 1.0, 0.0, 1.0, 1.0);
    assert!((z - prob).abs() < 1e-10);
}

#[test]
fn test_layer_5_multi_atom_field_coupling() {
    use coh_physics::field::CohField;
    
    let mut field = CohField::new(0.01); // Small coupling g
    
    let state_x = Hash32([0; 32]);
    let bit = CohBit {
        from_state: state_x,
        to_state: Hash32([1; 32]),
        transition_id: "test".to_string(),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(100, 1),
        spend: Rational64::new(0, 1),
        defect: Rational64::new(0, 1),
        delta_hat: Rational64::new(0, 1),
        utility: 10.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([2; 32]),
        state: CohBitState::Superposed,
    };

    let atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![bit.clone()],
        geometry: AtomGeometry {
            distance: Rational64::new(0, 1),
            curvature: 0.0,
            ricci_scalar: 0.0,
        },
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(0, 1),
        },
        receipt_chain: vec![],
    };

    field.atoms.push(atom.clone());
    
    let mut atom2 = atom.clone();
    atom2.state_hash = Hash32([1; 32]);
    field.atoms.push(atom2); // Two atoms at different states

    // Interaction cost should be g * neighbor_density = 0.01 * 100 = 1.0
    let cost = field.interaction_cost(&field.atoms[0], &bit);
    assert!((cost - 1.0).abs() < 1e-10);
    
    // Step the field
    let transitions = field.step(1.0);
    assert_eq!(transitions.len(), 2);
}

#[test]
fn test_layer_4_coh_yang_mills_curvature() {
    let mut curvature = YangMillsCurvature {
        dim: 3,
        f: [[[0.0; 8]; 4]; 4],
    };
    
    // Set some non-Abelian constraint conflict
    curvature.f[0][1][0] = 1.0; // conflict between crypto (a=0) in t-x plane
    curvature.f[0][1][1] = 2.0; // conflict in thermal (a=1)
    
    let density = curvature.action_density();
    // Sum F^2 = 1.0^2 + 2.0^2 = 5.0
    // Note: in a real trace, indices are antisymmetrized, so F_01 and F_10 would both exist.
    assert!(density >= 5.0);
}

#[test]
fn test_full_hierarchy_control_law() {
    let state_x = Hash32([0; 32]);
    let bit = CohBit {
        from_state: state_x,
        to_state: Hash32([1; 32]),
        transition_id: "test".to_string(),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(100, 1),
        spend: Rational64::new(0, 1),
        defect: Rational64::new(0, 1),
        delta_hat: Rational64::new(10, 1), // Geometric cost
        utility: 50.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([2; 32]),
        state: CohBitState::Superposed,
    };

    let atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![bit.clone()],
        geometry: AtomGeometry {
            distance: Rational64::new(0, 1),
            curvature: 0.0,
            ricci_scalar: 0.5,
        },
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(5, 1),
        },
        receipt_chain: vec![],
    };

    // Action J = delta_hat - utility + lambda * (ricci + gauge) - refresh
    // J = 10 - 50 + 1.0 * (0.5 + 5.0) - 5 = -39.5
    let action = atom.compute_action(&bit, 1.0, 5.0);
    assert!((action + 39.5).abs() < 1e-10);
    
    let optimal = atom.select_optimal_bit(1.0, 5.0);
    assert!(optimal.is_some());
}
