use coh_genesis::lemma_forge::{LemmaForge, DerivationKind};

fn main() {
    println!("--- CTRL-v1.3 Lemma Forge Benchmark ---");

    let cases = vec![
        (
            "v_post ≤ v_pre + defect + authority",
            "v_post + spend ≤ v_pre + defect + authority",
            DerivationKind::ArithmeticInequality,
        ),
        (
            "AxiomDeps Ac = AxiomDeps τ",
            "ConservativeCompression τ Ac",
            DerivationKind::InvariantExtraction,
        ),
        (
            "(x + y) * z = x * z + y * z",
            "x, y, z : ℝ",
            DerivationKind::RingEquality,
        ),
    ];

    let mut total_passed = 0;
    for (goal, context, expected_kind) in &cases {
        let plan = LemmaForge::plan(goal, context);
        let lemma = LemmaForge::synthesize(&plan);
        
        let kind_match = plan.derivation_kind == *expected_kind;
        let contains_lemma = lemma.contains("lemma");
        let contains_by = lemma.contains(":= by");
        
        let success = kind_match && contains_lemma && contains_by;

        println!("Goal: {}", goal);
        println!("  Kind:   {:?}", plan.derivation_kind);
        println!("  Lemma:  \n{}", lemma);
        println!("  Result: {}", if success { "OK" } else { "FAIL" });

        if success {
            total_passed += 1;
        }
    }

    println!("\nSummary: {}/{} cases passed.", total_passed, cases.len());
    let accuracy = total_passed as f32 / cases.len() as f32;
    println!("LemmaForgeSynthesisAccuracy: {:.2}", accuracy);
}
