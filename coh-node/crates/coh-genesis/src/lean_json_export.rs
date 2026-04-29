//! Lean JSON Export for Search Results
//!
//! Replaces stdout scraping with structured JSON output.
//! A 20-line Lean metaprogram that exports search results as JSON.
//! This eliminates an entire class of parsing bugs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio, Child};
use std::time::{Duration, Instant};
use std::thread;

/// Kill a process and all its children (tree-kill)
fn kill_process_tree(mut child: Child) {
    #[cfg(windows)]
    {
        let pid = child.id();
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

/// Schema for JSON search results from Lean
pub const LEAN_SEARCH_JSON_SCHEMA: &str = "coh.lean.search.v1";

/// Search result item from Lean JSON export
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeanSearchResult {
    /// Lemma name/identifier
    pub name: String,
    /// Lemma type signature
    #[serde(rename = "type")]
    pub lemma_type: String,
    /// Source file location
    pub location: Option<String>,
    /// Whether the lemma is reducible
    pub reducible: Option<bool>,
}

/// JSON output structure from Lean metaprogram
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeanSearchResults {
    /// Schema identifier
    pub schema: String,
    /// Version
    pub version: String,
    /// Query that was searched
    pub query: String,
    /// Number of results
    pub count: usize,
    /// Results array
    pub results: Vec<LeanSearchResult>,
    /// Errors if any
    pub errors: Option<String>,
}

/// Execute Lean JSON search (replaces #find stdout scraping)
pub fn execute_lean_json_search(
    project_path: &Path,
    lake_cmd: &str,
    query: &str,
    timeout_secs: Option<u64>,
) -> LeanSearchResults {
    // Use unique temp file for parallel tests
    let thread_id = format!("{:?}", thread::current().id())
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let temp_file_name = format!("_json_search_{}.lean", thread_id);
    let temp_file = project_path.join(&temp_file_name);

    // Generate Lean code that outputs JSON instead of human-readable text
    // We'll use a simplified Lean metaprogram for now
    let lean_code = format!(
        r#"import Mathlib.Tactic.Find
open Lean Elab Term Meta

#eval do
  let results <- findFrontend "{}"
  let mut results_json := []
  for r in results do
    results_json := results_json.concat s!"{{\"name\":\"{{r.1}}\", \"type\":\"{{r.2}}\"}}"
  
  let json := s!"{{\"schema\": \"{}\", \"version\": \"1.0.0\", \"query\": \"{}\", \"count\": {{results.length}}, \"results\": [{{", ".join results_json}}]}}"
  IO.println json
"#,
        query.replace('\\', "\\\\").replace('"', "\\\""),
        LEAN_SEARCH_JSON_SCHEMA,
        query.replace('\\', "\\\\").replace('"', "\\\"")
    );

    // Write temp file
    if std::fs::write(&temp_file, lean_code).is_err() {
        return LeanSearchResults {
            schema: LEAN_SEARCH_JSON_SCHEMA.to_string(),
            version: "1.0.0".to_string(),
            query: query.to_string(),
            count: 0,
            results: vec![],
            errors: Some("Failed to write temp file".to_string()),
        };
    }

    // Execute Lean with optional timeout
    let mut child = match Command::new(lake_cmd)
        .args(["env", "lean", &temp_file_name])
        .current_dir(project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_file(&temp_file);
                return LeanSearchResults {
                    schema: LEAN_SEARCH_JSON_SCHEMA.to_string(),
                    version: "1.0.0".to_string(),
                    query: query.to_string(),
                    count: 0,
                    results: vec![],
                    errors: Some(format!("Spawn error: {}", e)),
                };
            }
        };

    let timeout = Duration::from_secs(timeout_secs.unwrap_or(60));
    let start = Instant::now();
    let mut timed_out = false;

    let output = loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                break child.wait_with_output();
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    println!("WARNING: Mathlib search timed out after {}s: {}", timeout.as_secs(), query);
                    kill_process_tree(child);
                    timed_out = true;
                    // Return a dummy successful output with empty buffers
                    // because we handle timed_out later.
                    break Ok(std::process::Output {
                        status: Default::default(),
                        stdout: vec![],
                        stderr: vec![],
                    });
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                let _ = std::fs::remove_file(&temp_file);
                return LeanSearchResults {
                    schema: LEAN_SEARCH_JSON_SCHEMA.to_string(),
                    version: "1.0.0".to_string(),
                    query: query.to_string(),
                    count: 0,
                    results: vec![],
                    errors: Some(format!("Wait error: {}", e)),
                };
            }
        }
    };

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);

    match output {
        Ok(out) => {
            if timed_out {
                return LeanSearchResults {
                    schema: LEAN_SEARCH_JSON_SCHEMA.to_string(),
                    version: "1.0.0".to_string(),
                    query: query.to_string(),
                    count: 0,
                    results: vec![],
                    errors: Some(format!("Timeout after {}s", timeout.as_secs())),
                };
            }
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            // Try to parse JSON response
            if let Ok(results) = serde_json::from_str::<LeanSearchResults>(&stdout) {
                results
            } else {
                LeanSearchResults {
                    schema: LEAN_SEARCH_JSON_SCHEMA.to_string(),
                    version: "1.0.0".to_string(),
                    query: query.to_string(),
                    count: 0,
                    results: vec![],
                    errors: Some(format!("Parse error: {}. Stderr: {}", stdout, stderr)),
                }
            }
        }
        Err(e) => LeanSearchResults {
            schema: LEAN_SEARCH_JSON_SCHEMA.to_string(),
            version: "1.0.0".to_string(),
            query: query.to_string(),
            count: 0,
            results: vec![],
            errors: Some(format!("Execution error: {}", e)),
        },
    }
}

/// Export search results to a JSON file
pub fn export_search_json(
    project_path: &Path,
    lake_cmd: &str,
    query: &str,
    output_file: &Path,
    timeout_secs: Option<u64>,
) -> Result<LeanSearchResults, String> {
    let results = execute_lean_json_search(project_path, lake_cmd, query, timeout_secs);
    let json = serde_json::to_string_pretty(&results)
        .map_err(|e| format!("JSON serialization error: {}", e))?;
    std::fs::write(output_file, json).map_err(|e| format!("File write error: {}", e))?;
    Ok(results)
}

/// Batch search for multiple queries
pub fn batch_search_json(
    project_path: &Path,
    lake_cmd: &str,
    queries: &[String],
    output_dir: &Path,
    timeout_secs: Option<u64>,
) -> HashMap<String, LeanSearchResults> {
    let mut results = HashMap::new();
    for query in queries {
        let output_file = output_dir.join(format!("search_{}.json", query.replace(' ', "_")));
        match export_search_json(project_path, lake_cmd, query, &output_file, timeout_secs) {
            Ok(r) => { results.insert(query.clone(), r); }
            Err(e) => {
                results.insert(query.clone(), LeanSearchResults {
                    schema: LEAN_SEARCH_JSON_SCHEMA.to_string(),
                    version: "1.0.0".to_string(),
                    query: query.clone(),
                    count: 0,
                    results: vec![],
                    errors: Some(e),
                });
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_structure() {
        let result = LeanSearchResult {
            name: "add_assoc".to_string(),
            lemma_type: "∀ {α} [Add α] (a b c : α), (a + b) + c = a + (b + c)".to_string(),
            location: Some("Mathlib/Algebra/Basic.lean:123".to_string()),
            reducible: Some(true),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("add_assoc"));
    }

    #[test]
    fn test_timeout_logic() {
        // We simulate a slow command using 'ping' on Windows or 'sleep' on Unix
        let cmd = if cfg!(windows) { "ping" } else { "sleep" };
        let args = if cfg!(windows) { vec!["127.0.0.1", "-n", "5"] } else { vec!["5"] };
        
        let start = Instant::now();
        let mut child = Command::new(cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn slow command");

        let timeout = Duration::from_secs(1);
        let mut timed_out = false;

        loop {
            match child.try_wait() {
                Ok(Some(_status)) => break,
                Ok(None) => {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        timed_out = true;
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(_) => break,
            }
        }

        assert!(timed_out);
        assert!(start.elapsed() >= timeout);
        assert!(start.elapsed() < Duration::from_secs(5));
    }
}
