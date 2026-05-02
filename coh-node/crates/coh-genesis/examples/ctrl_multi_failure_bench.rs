use coh_genesis::lean_error::{classify_lean_error, LeanErrorKind};
use coh_genesis::repair::{choose_repair_action, RepairAction};

fn main() {
    println!("--- CTRL-v1.1 Multi-Failure Benchmark ---");

    let cases = vec![
        (
            "unknown identifier 'Real'",
            LeanErrorKind::UnknownIdentifier,
            RepairAction::AddImport("Mathlib.Tactic".into()),
        ),
        (
            "unsolved goals\n⊢ 1 + 1 = 2",
            LeanErrorKind::UnsolvedGoals,
            RepairAction::DecomposeGoals,
        ),
        (
            "type mismatch, expected 'Real' but got 'Nat'",
            LeanErrorKind::TypeMismatch,
            RepairAction::TypeAlign,
        ),
        (
            "rewrite tactic failed, did not find instance of pattern",
            LeanErrorKind::RewriteFailed,
            RepairAction::RewriteAlternate,
        ),
        (
            "heartbeat exhausted, timeout",
            LeanErrorKind::Timeout,
            RepairAction::ReduceSearch,
        ),
        (
            "error: sorry used in proof",
            LeanErrorKind::UsesForbiddenShortcut,
            RepairAction::RejectPolicyViolation,
        ),
    ];

    let mut passed = 0;
    for (raw, expected_kind, expected_action) in &cases {
        let kind = classify_lean_error(raw);
        let action = choose_repair_action(kind);

        let kind_ok = kind == *expected_kind;
        let action_ok = action == *expected_action;

        println!("Input: \"{}\"", raw);
        println!("  Classified: {:?} [{}]", kind, if kind_ok { "OK" } else { "FAIL" });
        println!("  Action:     {:?} [{}]", action, if action_ok { "OK" } else { "FAIL" });

        if kind_ok && action_ok {
            passed += 1;
        }
    }

    println!("\nSummary: {}/{} cases passed.", passed, cases.len());
    let accuracy = passed as f32 / cases.len() as f32;
    println!("FailureClassificationAccuracy: {:.2}", accuracy);
}
