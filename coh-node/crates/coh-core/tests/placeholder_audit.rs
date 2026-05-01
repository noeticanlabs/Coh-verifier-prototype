// fixture_only: allow_mock
use std::{fs, path::Path};

const BANNED_PATTERNS: &[&str] = &[
    "0xAA", // fixture_only: allow_mock
    "0xBB", // fixture_only: allow_mock
    "0xCC", // fixture_only: allow_mock
    "Hash32::zero()", // fixture_only: allow_mock
    "Signature::empty()", // fixture_only: allow_mock
    "mock", // fixture_only: allow_mock
    "dummy", // fixture_only: allow_mock
    "placeholder", // fixture_only: allow_mock
    "fake", // fixture_only: allow_mock
    "stub", // fixture_only: allow_mock
    "default_cert", // fixture_only: allow_mock
];

fn scan_file(path: &Path, failures: &mut Vec<String>) {
    let Ok(src) = fs::read_to_string(path) else { return };

    // Check for file-level fixture allowance in first 5 lines
    let file_fixture_allowed = src.lines().take(5).any(|l| {
        let lower = l.to_lowercase();
        lower.contains("fixture_only") && lower.contains("allow_mock")
    });
    if file_fixture_allowed {
        return;
    }

    for (line_no, line) in src.lines().enumerate() {
        let lower = line.to_lowercase();
        // Skip line if it has surgical allowance
        if lower.contains("fixture_only") && lower.contains("allow_mock") {
            continue;
        }

        for pattern in BANNED_PATTERNS {
            if lower.contains(&pattern.to_lowercase()) {
                failures.push(format!(
                    "{}:{} contains banned placeholder pattern `{}`: {}",
                    path.display(),
                    line_no + 1,
                    pattern,
                    line.trim()
                ));
            }
        }
    }
}

fn scan_dir(path: &Path, failures: &mut Vec<String>) {
    if !path.exists() { return; }
    
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap().to_string_lossy();
            if name == "target" || name == ".git" || name == ".gemini" || name == ".tempmediaStorage" 
                || name == "examples" || name == "tests" || name == "benches"
                || name == "coh-cli" || name == "coh-fuzz" || name == "coh-npe"
                || name == "coh-gccp" || name == "coh-time" || name == "coh-genesis" {
                continue;
            }
            scan_dir(&path, failures);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("rs" | "lean" | "toml" | "json")
        ) {
            scan_file(&path, failures);
        }
    }
}

#[test]
fn no_unapproved_mock_or_placeholder_data() { // fixture_only: allow_mock
    let mut failures = Vec::new();
    let mut root = Path::new(".");
    if root.join("crates").exists() {
        // We are already at workspace root
    } else if Path::new("../..").join("crates").exists() {
        root = Path::new("../..");
    }

    scan_dir(root, &mut failures);

    assert!(
        failures.is_empty(),
        "Mock/placeholder audit failed (No fake mustaches allowed):\n{}", // fixture_only: allow_mock
        failures.join("\n")
    );
}
