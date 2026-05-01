use std::process::{Command, Stdio, Child};
use std::io::{Write, BufReader, BufRead};
use std::path::Path;
use serde_json;
use crate::lean_json_export::{LeanSearchResults, LEAN_SEARCH_JSON_SCHEMA};

pub struct LeanWorker {
    child: Child,
}

impl LeanWorker {
    pub fn start(project_path: &Path, lake_cmd: &str) -> Result<Self, String> {
        let child = Command::new(lake_cmd)
            .args(["env", "lean", "--run", "persistent_worker.lean"])
            .current_dir(project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn Lean worker: {}", e))?;

        Ok(Self { child })
    }

    pub fn query(&mut self, query: &str) -> Result<LeanSearchResults, String> {
        let stdin = self.child.stdin.as_mut().ok_or("Failed to open stdin")?;
        writeln!(stdin, "{}", query).map_err(|e| format!("Failed to write to Lean worker: {}", e))?;
        stdin.flush().map_err(|e| format!("Failed to flush Lean worker: {}", e))?;

        let stdout = self.child.stdout.as_mut().ok_or("Failed to open stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| format!("Failed to read from Lean worker: {}", e))?;

        if line.is_empty() {
            return Err("Lean worker returned empty response".to_string());
        }

        serde_json::from_str::<LeanSearchResults>(&line)
            .map_err(|e| format!("Failed to parse Lean worker response: {}. Raw: {}", e, line))
    }

    pub fn stop(mut self) {
        if let Some(mut stdin) = self.child.stdin.take() {
            let _ = writeln!(stdin, "EXIT");
            let _ = stdin.flush();
        }
        let _ = self.child.wait();
    }
}
