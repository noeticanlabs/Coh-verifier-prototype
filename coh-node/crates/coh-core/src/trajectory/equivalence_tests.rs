use crate::trajectory::*;
use crate::types::Hash32;

#[test]
fn test_v3_distance_triangle_inequality() {
    let mut engine = TrajectoryEngine::new();

    let x = StateNode {
        hash: Hash32([1; 32]),
        potential: 100,
    };
    let y = StateNode {
        hash: Hash32([2; 32]),
        potential: 80,
    };
    let z = StateNode {
        hash: Hash32([3; 32]),
        potential: 60,
    };

    // x -> y with delta 10
    engine.add_transition(Transition {
        from: x.clone(),
        to: y.clone(),
        delta: 10,
        step_type: None,
    });

    // y -> z with delta 5
    engine.add_transition(Transition {
        from: y.clone(),
        to: z.clone(),
        delta: 5,
        step_type: None,
    });

    // Direct x -> z with delta 20 (suboptimal)
    engine.add_transition(Transition {
        from: x.clone(),
        to: z.clone(),
        delta: 20,
        step_type: None,
    });

    let d_xy = engine.compute_distance(x.hash, y.hash).unwrap();
    let d_yz = engine.compute_distance(y.hash, z.hash).unwrap();
    let d_xz = engine.compute_distance(x.hash, z.hash).unwrap();

    assert_eq!(d_xy, 10);
    assert_eq!(d_yz, 5);
    assert_eq!(d_xz, 15); // Dijkstra should find the 10+5 path

    // Triangle Inequality: d(x, z) <= d(x, y) + d(y, z)
    assert!(d_xz <= d_xy + d_yz);
}
