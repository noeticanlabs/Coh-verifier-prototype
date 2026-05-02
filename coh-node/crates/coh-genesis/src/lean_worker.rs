use std::process::{Command, Stdio, Child};
use std::io::{Write, BufReader, BufRead};
use std::path::{Path, PathBuf};
use serde_json;
use crate::lean_json_export::LeanSearchResults;

pub struct LeanWorker {
    child: Child,
    lake_cmd: String,
    project_path: PathBuf,
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

        Ok(Self { child, lake_cmd: lake_cmd.to_string(), project_path: project_path.to_path_buf() })
    }

    pub fn query(&mut self, query: &str) -> Result<LeanSearchResults, String> {
        let stdin = self.child.stdin.as_mut().ok_or("Failed to open stdin")?;
        writeln!(stdin, "SEARCH|{}", query).map_err(|e| format!("Failed to write to Lean worker: {}", e))?;
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

    pub fn verify_step(&mut self, goal: &str, tactic: &str) -> Result<serde_json::Value, String> {
        let stdin = self.child.stdin.as_mut().ok_or("Failed to open stdin")?;
        writeln!(stdin, "VERIFY|{}|{}", goal, tactic).map_err(|e| format!("Failed to write to Lean worker: {}", e))?;
        stdin.flush().map_err(|e| format!("Failed to flush Lean worker: {}", e))?;

        let stdout = self.child.stdout.as_mut().ok_or("Failed to open stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| format!("Failed to read from Lean worker: {}", e))?;

        if line.is_empty() {
            return Err("Lean worker returned empty response".to_string());
        }

        serde_json::from_str::<serde_json::Value>(&line)
            .map_err(|e| format!("Failed to parse Lean worker response: {}. Raw: {}", e, line))
    }

    pub fn try_tactic(&mut self, thm_name: &str, tactic: &str) -> Result<serde_json::Value, String> {
        let stdin = self.child.stdin.as_mut().ok_or("Failed to open stdin")?;
        writeln!(stdin, "TRY_TACTIC|{}|{}", thm_name, tactic).map_err(|e| format!("Failed to write to Lean worker: {}", e))?;
        stdin.flush().map_err(|e| format!("Failed to flush Lean worker: {}", e))?;

        let stdout = self.child.stdout.as_mut().ok_or("Failed to open stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| format!("Failed to read from Lean worker: {}", e))?;

        if line.is_empty() {
            return Err("Lean worker returned empty response".to_string());
        }

        serde_json::from_str::<serde_json::Value>(&line)
            .map_err(|e| format!("Failed to parse Lean worker response: {}. Raw: {}", e, line))
    }

    pub fn full_build_output(&mut self) -> Result<std::process::Output, String> {
        Command::new(&self.lake_cmd)
            .args(["build", "Coh"])
            .current_dir(&self.project_path)
            .output()
            .map_err(|e| format!("Failed to run lake build: {}", e))
    }

    pub fn full_build(&mut self) -> Result<String, String> {
        let output = self.full_build_output()?;

        if output.status.success() {
            Ok("SUCCESS".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(format!("FAILURE\nSTDOUT: {}\nSTDERR: {}", stdout, stderr))
        }
    }

    pub fn stop(mut self) {
        if let Some(mut stdin) = self.child.stdin.take() {
            let _ = writeln!(stdin, "EXIT");
            let _ = stdin.flush();
        }
        let _ = self.child.wait();
    }
}
