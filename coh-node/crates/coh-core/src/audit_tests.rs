#[cfg(test)]
mod tests {
    // fixture_only: allow_mock
use std::fs;
    use std::path::Path;

    #[test]
    fn placeholder_audit() {
        let src_dir = Path::new("src");
        scan_directory(src_dir).expect("Audit failed to scan directory");
    }

    fn scan_directory(dir: &Path) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Skip fixture modules and the audit test itself
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if file_name == "fixtures.rs" || file_name == "audit_tests.rs" || path.to_str().unwrap_or("").contains("fixtures/") {
                    continue;
                }
                scan_file(&path)?;
            }
        }
        Ok(())
    }

    fn scan_file(path: &Path) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        // Check for file-level fixture allowance in first 5 lines
        let file_fixture_allowed = lines.iter().take(5).any(|l| {
            let lower = l.to_lowercase();
            lower.contains("fixture_only") && lower.contains("allow_mock")
        });

        if file_fixture_allowed {
            return Ok(());
        }

        let mut errors = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            
            // Check for illegal placeholders
            let illegal = [
                ("0xAA", "Fixture ID constant"),
                ("0xCC", "Compression fixture constant"),
                ("Hash32([0; 32])", "Zero hash placeholder"),
                ("Signature(vec![0; 64])", "Empty signature placeholder"),
                ("Signature(vec![0;64])", "Empty signature placeholder"),
            ];

            for (pattern, reason) in illegal {
                if line.contains(pattern) {
                    // Check for line-level allowance
                    let lower = line.to_lowercase();
                    if !lower.contains("allow_mock") && !lower.contains("fixture_only") {
                        errors.push(format!(
                            "Line {}: Found illegal pattern '{}' ({})",
                            line_num, pattern, reason
                        ));
                    }
                }
            }
        }

        if !errors.is_empty() {
            panic!(
                "\nAudit failed for file: {:?}\n{}\nTo allow this in a core module, add '// fixture_only: allow_mock' to the line or file header.",
                path,
                errors.join("\n")
            );
        }

        Ok(())
    }
}
