// fixture_only: allow_mock
use coh_core::cohbit::CohBit;
use coh_core::atom::{CohAtom, CohGovernor, AtomGeometry, AtomMetabolism};
use coh_physics::CohSpinor;
use coh_physics::current::CoherenceCurrent;
use coh_physics::gauge::{CohGaugeField, YangMillsCurvature, WilsonLoopReceipt};
use coh_core::types::{Hash32, RvStatus};
use num_rational::Rational64;
use num_complex::Complex64;

#[test]
fn test_layer_1_cohbit_admissibility() {
    let bit = CohBit {
        from_state: Hash32([1; 32]),
        to_state: Hash32([2; 32]),
        action_hash: Hash32([0x22; 32]),
        projection_hash: Hash32([2; 32]),
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(10, 1),
        defect: Rational64::new(5, 1),
        delta_hat: Rational64::new(5, 1),
        utility: Rational64::from_integer(1),
        probability_soft: Rational64::from_integer(0),
        probability_exec: Rational64::from_integer(0),
        rv_status: RvStatus::Accept,
        receipt_hash: Hash32([3; 32]),
        bit_id: Hash32([0x11; 32]),
        signature: coh_core::types::Signature(vec![1; 64]),
        ..Default::default()
    }.finalize_hashes();

    assert_eq!(bit.margin(), Rational64::new(5, 1));
    assert!(bit.executable());
}

#[test]
fn test_layer_2_coh_atom_evolution() {
    let state_x = Hash32([1; 32]);
    let state_y = Hash32([2; 32]);
    
    let bit = CohBit {
        from_state: state_x,
        to_state: state_y,
        action_hash: Hash32([0x22; 32]),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(5, 1),
        defect: Rational64::new(2, 1),
        delta_hat: Rational64::new(2, 1),
        utility: Rational64::from_integer(10),
        rv_status: RvStatus::Accept,
        bit_id: Hash32([0x11; 32]),
        signature: coh_core::types::Signature(vec![1; 64]),
        ..Default::default()
    }.finalize_hashes();

    let mut gov = CohGovernor {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        geometry: AtomGeometry {
            distance: Rational64::new(0, 1),
            curvature: 0.0,
            ricci_scalar: 0.1,
        },
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(10, 1),
        },
    };

    let success = gov.evolve(&bit);
    assert!(success);
    assert_eq!(gov.state_hash, state_y);
    assert_eq!(gov.valuation, Rational64::new(90, 1));
    assert_eq!(gov.metabolism.budget, Rational64::new(1005, 1));
}

#[test]
fn test_layer_3_coh_spinor_current() {
    let val: f64 = 1.0;
    let psi = CohSpinor::new(
        Complex64::new(val.sqrt(), 0.0),
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
    );
    
    assert!((psi.density() - val).abs() < 1e-10);
    
    let current = CoherenceCurrent::compute(&psi);
    assert!((current.j0 - val).abs() < 1e-10);

    let g_base = [[1.0, 0.0, 0.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [0.0, 0.0, 0.0, -1.0]];
    let g_eff = current.effective_metric_coupling(g_base, 0.0, 0.1, 0.0);
    assert!((g_eff[0][0] - 1.1).abs() < 1e-10);
}

#[test]
fn test_layer_4_locked_yang_mills_curvature() {
    let mut gauge = CohGaugeField::new(3);
    
    // Set non-commuting links: A_t in Pauli-x, A_x in Pauli-y
    gauge.connection[0][0] = 0.1; // A_0^1
    gauge.connection[1][1] = 0.1; // A_1^2
    
    let f01 = gauge.compute_curvature(0, 1);
    
    // Non-Abelian term: [0.1 Tx, 0.1 Ty] = 0.01 i Tz
    // f01[2] should include this bracket term
    assert!(f01[2].abs() > 0.005);
    
    let mut curvature = YangMillsCurvature {
        dim: 3,
        f: [[[0.0; 8]; 4]; 4],
    };
    curvature.f[0][1][2] = f01[2];
    
    let density = curvature.action_density();
    // Tr(F^2) = Tr( (f Tx)^2 ) = 1/2 f^2
    assert!(density > 0.0);
}

#[test]
fn test_layer_5_multi_atom_field_coupling() {
    use coh_physics::field::CohField;
    let mut field = CohField::new(0.01);
    let state_x = Hash32([1; 32]);
    let bit = CohBit {
        from_state: state_x,
        to_state: Hash32([2; 32]),
        action_hash: Hash32([0x22; 32]),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(100, 1),
        spend: Rational64::new(0, 1),
        defect: Rational64::new(0, 1),
        delta_hat: Rational64::new(0, 1),
        utility: Rational64::from_integer(10),
        rv_status: RvStatus::Accept,
        bit_id: Hash32([0x11; 32]),
        signature: coh_core::types::Signature(vec![1; 64]),
        ..Default::default()
    }.finalize_hashes();

    field.governors.push(CohGovernor {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        ..Default::default()
    });

    let cost = field.interaction_cost(&field.governors[0], &bit);
    assert!(cost >= 0.0);
}

#[test]
fn test_layer_6_path_integral_weighting() {
    use coh_core::trajectory::path_integral::CohHistory;
    let state_x = Hash32([1; 32]);
    let bit = CohBit {
        from_state: state_x,
        to_state: Hash32([2; 32]),
        action_hash: Hash32([0x22; 32]),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(5, 1),
        defect: Rational64::new(0, 1),
        delta_hat: Rational64::new(2, 1),
        utility: Rational64::from_integer(10),
        rv_status: RvStatus::Accept,
        bit_id: Hash32([0x11; 32]),
        signature: coh_core::types::Signature(vec![1; 64]),
        ..Default::default()
    }.finalize_hashes();

    let history = CohHistory { steps: vec![bit.clone()] };
    let gov = CohGovernor {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        ..Default::default()
    };

    let prob = history.path_probability(&gov, 1.0, 0.0, 1.0, 1.0);
    assert!(prob > 0.0);
}

#[test]
fn test_layer_7_wilson_loop_holonomy() {
    use coh_physics::gauge::{CohGaugeField, WilsonLoopReceipt};
    use coh_core::trajectory::path_integral::CohHistory;
    
    let mut gauge = CohGaugeField::new(3);
    gauge.connection[0][0] = 0.1; // Total rotation phase
    
    let bit = CohBit {
        from_state: Hash32([1; 32]),
        to_state: Hash32([2; 32]),
        action_hash: Hash32([0x22; 32]),
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(100, 1),
        rv_status: RvStatus::Accept,
        bit_id: Hash32([0x11; 32]),
        signature: coh_core::types::Signature(vec![1; 64]),
        ..Default::default()
    }.finalize_hashes();

    let history = CohHistory { steps: vec![bit.clone(); 10] };

    let holonomy = WilsonLoopReceipt::compute_holonomy(&history, &gauge);
    assert!(holonomy >= 0.0);
    
    let receipt = WilsonLoopReceipt {
        path_hash: "test".to_string(),
        holonomy_trace: holonomy,
        curvature_sum: 0.0,
        constraint_residual: 0.0,
        bianchi_residual: 0.0,
        ym_energy: 0.0,
    };
    
    assert!(receipt.holonomy_trace >= 0.0);
}
