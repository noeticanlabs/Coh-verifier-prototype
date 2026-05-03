use coh_genesis::equivalence_hunter::{EquivalenceHunter, EquivalenceKind};
use std::fs::OpenOptions;
use std::io::Write;
use serde_json::json;

fn main() {
    println!("--- CTRL-v1.3 Equivalence Hunter Benchmark ---");

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

    // Ensure directory exists
    std::fs::create_dir_all("reports").ok();
    let log_path = "reports/ctrl_equivalence_bench.ndjson";
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Failed to open equivalence log");

    let mut total_passed = 0;
    for (source, target, expected_kind) in &cases {
        let diagnosis = EquivalenceHunter::hunt(source, target);
        let success = diagnosis.kind == *expected_kind;

        let log_entry = json!({
            "source": source,
            "target": target,
            "detected_kind": diagnosis.kind,
            "expected_kind": expected_kind,
            "success": success,
        });

        writeln!(file, "{}", log_entry.to_string()).expect("Failed to write log entry");

        println!("Source: {}", source);
        println!("  Target: {}", target);
        println!("  Kind:   {:?}", diagnosis.kind);
        println!("  Result: {}", if success { "OK" } else { "FAIL" });

        if success {
            total_passed += 1;
        }
    }

    println!("\nSummary: {}/{} cases passed.", total_passed, cases.len());
}
