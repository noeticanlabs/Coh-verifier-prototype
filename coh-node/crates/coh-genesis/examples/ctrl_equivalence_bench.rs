use coh_genesis::equivalence_hunter::{EquivalenceHunter, EquivalenceKind};

fn main() {
    println!("--- CTRL-v1.4 Equivalence Hunter Benchmark ---");

    let cases = vec![
        (
            "trajectory_bisimulation_atom",
            "SummaryAtom Ac",
            EquivalenceKind::CompressionEquivalence,
        ),
        (
            "hamiltonian_valuation_mapping",
            "Valuation V",
            EquivalenceKind::Isomorphism,
        ),
        (
            "gauge_potential_phi",
            "Gauge field A'",
            EquivalenceKind::GaugeEquivalence,
        ),
        (
            "energy_normalization",
            "E + 0",
            EquivalenceKind::NormalFormEquivalence,
        ),
    ];

    let mut total_passed = 0;
    for (source, target, expected_kind) in &cases {
        let diagnosis = EquivalenceHunter::hunt(source, target);
        let success = diagnosis.kind == *expected_kind;

        println!("Source: {}", source);
        println!("  Target: {}", target);
        println!("  Kind:   {:?}", diagnosis.kind);
        println!("  Result: {}", if success { "OK" } else { "FAIL" });

        if success {
            total_passed += 1;
        }
    }

    println!("\nSummary: {}/{} cases passed.", total_passed, cases.len());
    let accuracy = total_passed as f32 / cases.len() as f32;
    println!("EquivalenceDetectionAccuracy: {:.2}", accuracy);
}
