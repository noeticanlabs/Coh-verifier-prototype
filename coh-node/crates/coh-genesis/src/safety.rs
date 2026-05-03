/// Safety module for enforcing the "No-Bluff" protocol in Lean 4 proof generation.
///
/// Prevents the generation or acceptance of candidates that use forbidden shortcuts
/// like `sorry`, `admit`, or `axiom`.

pub fn contains_forbidden_shortcut(s: &str) -> bool {
    let lower = s.to_lowercase();
    // Check for common Lean 4 proof shortcuts
    lower.contains("sorry") || 
    lower.contains("admit") || 
    lower.contains("axiom") ||
    lower.contains("cheat") // Sometimes used in informal scripts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forbidden_shortcuts() {
        assert!(contains_forbidden_shortcut("exact sorry"));
        assert!(contains_forbidden_shortcut("admit"));
        assert!(contains_forbidden_shortcut("axiom my_axiom : True"));
        assert!(!contains_forbidden_shortcut("simp; ring"));
    }
}
