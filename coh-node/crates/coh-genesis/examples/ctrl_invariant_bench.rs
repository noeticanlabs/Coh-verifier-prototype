use coh_genesis::invariant_hunter::{InvariantHunter, InvariantKind};

fn main() {
    println!("--- CTRL-v1.2 Invariant Hunter Benchmark ---");

    let cases = vec![
        (
            "lorentz_summary_equivalent_to_raw_trajectory",
            "LorentzInvariantSummary(Ac)",
            "ConservativeCompression τ Ac",
            vec![InvariantKind::LorentzInvariance], // Expected missing
        ),
        (
            "trajectory_commit_stability",
            "TrajectoryAdmissible τ",
            "Trajectory τ",
            vec![InvariantKind::CommitInequality], // Expected missing
        ),
        (
            "summary_atom_lineage_lock",
            "SummaryAtom(Ac)",
            "Trajectory τ",
            vec![InvariantKind::LineageLock, InvariantKind::EndpointPreservation, InvariantKind::MarginConservativity], // Expected missing
        ),
    ];

    let mut total_passed = 0;
    for (name, stmt, ctx, expected_missing) in &cases {
        let diagnosis = InvariantHunter::hunt(name, stmt, ctx);
        
        let all_missing_found = expected_missing.iter().all(|m| diagnosis.missing.contains(m));
        let no_extra_missing = diagnosis.missing.iter().all(|m| expected_missing.contains(m));

        let success = all_missing_found && no_extra_missing;
        
        println!("Theorem: {}", name);
        println!("  Context: {}", ctx);
        println!("  Missing: {:?}", diagnosis.missing);
        println!("  Result:  {}", if success { "OK" } else { "FAIL" });

        if success {
            total_passed += 1;
        }
    }

    println!("\nSummary: {}/{} cases passed.", total_passed, cases.len());
    let accuracy = total_passed as f32 / cases.len() as f32;
    println!("InvariantDetectionAccuracy: {:.2}", accuracy);
}
