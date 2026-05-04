use clap::{Parser, Subcommand, ValueEnum};
use coh_core::build_slab::build_slab;
use coh_core::types::*;
use coh_core::verify_chain::verify_chain;
use coh_core::verify_micro;
use coh_core::verify_slab_envelope;
use coh_genesis::*;
use num_rational::Rational64;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process;

#[derive(Parser)]
#[command(name = "coh-validator")]
#[command(about = "Coh Constraint Verifier Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify a single micro-receipt in isolation
    VerifyMicro { input: String },
    /// Verify a chain of micro-receipts from a JSONL file
    VerifyChain { input: String },
    /// Build a slab-receipt from a chain of micro-receipts
    BuildSlab {
        input: String,
        #[arg(long, short)]
        out: String,
    },
    /// Verify a standalone slab-receipt
    VerifySlab { input: String },
    /// Run a single GMI Governor step
    GmiStep {
        #[arg(long)]
        proposal_id: String,
        #[arg(long)]
        content: String,
        #[arg(long, default_value = "100/1")]
        distance: String,
    },
    /// Run the Wildness Seeker sweep
    WildnessSweep {
        #[arg(long, default_value = "10")]
        steps: usize,
    },
    /// Run the NPE Loop (PhaseLoom enabled)
    NpeLoop {
        #[arg(long, default_value = "5")]
        iterations: usize,
    },
    /// Repair a Lean theorem using CTRL loop
    Repair {
        /// Path to the Lean project (containing lake-manifest.json)
        #[arg(long)]
        project: String,
        /// Theorem name to repair
        #[arg(long)]
        theorem: String,
        /// Candidate tactics to try (comma-separated)
        #[arg(long, default_value = "sorry")]
        tactics: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::VerifyMicro { input } => {
            let wire: MicroReceiptWire = match load_json(&input) {
                Ok(w) => w,
                Err(e) => exit_with_error(
                    format!("Failed to load micro-receipt from {}: {}", input, e),
                    2,
                    cli.format,
                ),
            };
            let res = verify_micro(wire);
            output_result(res, cli.format);
        }
        Commands::VerifyChain { input } => {
            let receipts = match load_jsonl(&input) {
                Ok(r) => r,
                Err(e) => exit_with_error(
                    format!("Failed to load chain from {}: {}", input, e),
                    2,
                    cli.format,
                ),
            };
            let res = verify_chain(receipts);
            output_result(res, cli.format);
        }
        Commands::BuildSlab { input, out } => {
            let receipts = match load_jsonl(&input) {
                Ok(r) => r,
                Err(e) => exit_with_error(
                    format!("Failed to load source chain from {}: {}", input, e),
                    2,
                    cli.format,
                ),
            };
            let mut res = build_slab(receipts);
            if res.decision == Decision::SlabBuilt {
                if let Some(ref slab) = res.slab {
                    if let Err(e) = save_json(&out, &slab) {
                        exit_with_error(
                            format!("Failed to save slab to {}: {}", out, e),
                            3,
                            cli.format,
                        );
                    }
                    res.output = Some(out.clone());
                }
                output_result(res, cli.format);
            } else {
                let exit_code = if let Some(code) = &res.code {
                    match code {
                        RejectCode::RejectChainDigest | RejectCode::RejectStateHashLink => 4,
                        RejectCode::RejectSchema if res.message.contains("Index discontinuity") => {
                            4
                        }
                        _ => 1,
                    }
                } else {
                    1
                };
                output_result_with_exit(res, cli.format, exit_code);
            }
        }
        Commands::VerifySlab { input } => {
            let wire: SlabReceiptWire = match load_json(&input) {
                Ok(w) => w,
                Err(e) => exit_with_error(
                    format!("Failed to load slab-receipt from {}: {}", input, e),
                    2,
                    cli.format,
                ),
            };
            let res = verify_slab_envelope(wire);
            output_result(res, cli.format);
        }
        Commands::GmiStep {
            proposal_id,
            content,
            distance,
        } => {
            let mut gov = setup_mock_governor();
            let dist_rational = parse_rational(&distance);

            let (success, trace) = gov.step(
                &proposal_id,
                &content,
                dist_rational,
                Rational64::new(10, 1),
                Rational64::new(1, 1),
                FormalStatus::ClosedNoSorry,
            );

            if cli.format == Format::Json {
                println!("{}", serde_json::to_string_pretty(&trace).unwrap());
            } else {
                println!("GMI Step: {}", if success { "SUCCESS" } else { "REJECTED" });
                println!("Step ID: {}", trace.step_id);
                for event in trace.events {
                    println!("  - {}", event);
                }
            }
        }
        Commands::WildnessSweep { steps } => {
            println!("Running Wildness Sweep ({} steps per level)...", steps);
            let results = run_wildness_sweep(&standard_levels(), steps, 42);

            if cli.format == Format::Json {
                println!("{}", serde_json::to_string_pretty(&results).unwrap());
            } else {
                print_summary(&results);
            }
        }
        Commands::NpeLoop { iterations } => {
            println!("Running NPE Loop ({} iterations)...", iterations);
            let mut gov = setup_mock_governor();

            for i in 0..iterations {
                let pid = format!("prop-{}", i);
                let (success, _) = gov.step(
                    &pid,
                    "npe-loop-content",
                    Rational64::new(100, 1),
                    Rational64::new(10, 1),
                    Rational64::new(1, 1),
                    FormalStatus::BuildPassedWithSorry,
                );
                println!(
                    "  Iteration {}: {}",
                    i,
                    if success { "COMMITTED" } else { "REJECTED" }
                );
            }
        }
        Commands::Repair {
            project,
            theorem,
            tactics,
        } => {
            use coh_genesis::ctrl::CtrlLoop;
            use std::path::PathBuf;

            println!(
                "Running CTRL repair on theorem '{}' in project '{}'",
                theorem, project
            );

            let project_path = PathBuf::from(&project);
            if !project_path.exists() {
                exit_with_error(format!("Project not found: {}", project), 1, cli.format);
            }

            // Parse tactics
            let tactic_list: Vec<&str> = tactics.split(',').collect();
            println!("Trying tactics: {:?}", tactic_list);

            // Create CTRL loop
            let mut ctrl = match CtrlLoop::new(project_path) {
                Ok(c) => c,
                Err(e) => {
                    exit_with_error(format!("Failed to start CTRL loop: {}", e), 1, cli.format)
                }
            };

            // Run repair
            let result = ctrl.repair_theorem(&theorem, tactic_list);

            match result {
                Ok(res) => {
                    if cli.format == Format::Json {
                        // Manual JSON output - CtrlResult may not derive Serialize
                        let json = serde_json::json!({
                            "theorem": res.theorem,
                            "tactic": res.tactic,
                            "proof_hash": res.proof_hash,
                            "success": res.success,
                            "error_kind": res.error_kind,
                            "cohbit_candidate": res.cohbit_candidate.as_ref().map(|c| {
                                serde_json::json!({
                                    "admissible": c.admissible,
                                    "decision": c.decision(),
                                    "candidate_hash": c.receipt.candidate_hash
                                })
                            }),
                            "cohbit_receipt": res.cohbit_receipt.as_ref().map(|r| {
                                serde_json::json!({
                                    "theorem_name": r.theorem_name,
                                    "theorem_hash_pre": r.theorem_hash_pre,
                                    "theorem_hash_post": r.theorem_hash_post,
                                    "spend": r.spend,
                                    "sequence_accumulator": r.sequence_accumulator
                                })
                            })
                        });
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    } else {
                        println!("=== CTRL Repair Result ===");
                        println!("Theorem: {}", res.theorem);
                        println!("Tactic: {}", res.tactic);
                        println!("Success: {}", res.success);
                        println!("Proof Hash: {}", res.proof_hash);
                        if res.success {
                            if let Some(ref candidate) = res.cohbit_candidate {
                                println!("=== CohBit Candidate ===");
                                println!("Admissible: {}", candidate.admissible);
                                println!("Decision: {}", candidate.decision());
                                println!("Receipt Hash: {}", candidate.receipt.candidate_hash);
                            }
                            if let Some(ref receipt) = res.cohbit_receipt {
                                println!("=== CohBit Receipt ===");
                                println!("  Theorem: {}", receipt.theorem_name);
                                println!("  Pre Hash: {}", receipt.theorem_hash_pre);
                                println!("  Post Hash: {}", receipt.theorem_hash_post);
                                println!("  Spend: {}", receipt.spend);
                                println!("  Sequence: {}", receipt.sequence_accumulator);
                            }
                        } else {
                            println!("Error Kind: {:?}", res.error_kind);
                        }
                    }
                }
                Err(e) => {
                    exit_with_error(format!("Repair failed: {}", e), 1, cli.format);
                }
            }
        }
    }
}

fn parse_rational(s: &str) -> Rational64 {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 2 {
        let n = parts[0].parse::<i64>().unwrap_or(0);
        let d = parts[1].parse::<i64>().unwrap_or(1);
        Rational64::new(n, d)
    } else {
        Rational64::new(s.parse::<i64>().unwrap_or(0), 1)
    }
}

fn setup_mock_governor() -> GmiGovernor {
    let npe = NpeKernel {
        state: NpeState::new(NpeConfig::default()),
        governing_state: NpeGoverningState {
            disorder: 1000,
            accumulated_cost: 0,
            wildness: 1.0,
            queue_depth: 0,
            memory_warmth: 0.5,
        },
        budget: NpeBudget::default(),
    };
    let rv = RvKernel {
        state: RvGoverningState {
            valuation: 5000,
            verified_spend: 0,
            allowable_defect: 1000,
            queue_depth: 0,
            ledger_tip: Hash32([0; 32]),
        },
        budget: ProtectedRvBudget::default(),
        mode: ToolAuthorityMode::Certification,
    };
    let phaseloom = PhaseLoomKernel {
        state: PhaseLoomState::default(),
        budget: PhaseLoomBudget::default(),
    };
    let env = EnvironmentalEnvelope {
        power_mj: None,
        thermal_headroom_c: None,
        wallclock_ms: 1000,
        hardware_available: true,
        network_allowed: false,
    };
    let system = SystemReserve {
        halt_available: true,
        logging_ops: 100,
        ledger_append_ops: 100,
        recovery_ops: 10,
        scheduler_ticks: 1000,
    };

    GmiGovernor::new(npe, rv, phaseloom, env, system, None)
}

fn load_json<T: serde::de::DeserializeOwned>(path: &str) -> anyhow::Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let val = serde_json::from_reader(reader)?;
    Ok(val)
}

fn load_jsonl<T: serde::de::DeserializeOwned>(path: &str) -> anyhow::Result<Vec<T>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut results = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();
        // Overbuilt Parser: Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        match serde_json::from_str::<T>(trimmed) {
            Ok(val) => results.push(val),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Line {}: JSON parsing failed: {}",
                    i + 1,
                    e
                ))
            }
        }
    }
    if results.is_empty() {
        return Err(anyhow::anyhow!(
            "File is empty or contains no valid records"
        ));
    }
    Ok(results)
}

fn save_json<T: serde::Serialize>(path: &str, val: &T) -> anyhow::Result<()> {
    let mut file = File::create(path)?;
    let buf = serde_json::to_vec_pretty(val)?;
    file.write_all(&buf)?;
    Ok(())
}

fn output_result<T: serde::Serialize + DisplayResult>(res: T, format: Format) {
    let exit_code = if res.is_accept() { 0 } else { 1 };
    output_result_with_exit(res, format, exit_code);
}

fn output_result_with_exit<T: serde::Serialize + DisplayResult>(
    res: T,
    format: Format,
    exit_code: i32,
) {
    match format {
        Format::Json => {
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Format::Text => {
            print!("{}", res.to_text());
        }
    }
    process::exit(exit_code);
}

fn exit_with_error(err: String, code: i32, format: Format) -> ! {
    match format {
        Format::Json => {
            let msg = serde_json::json!({
                "decision": "REJECT",
                "code": "RejectNumericParse",
                "message": err
            });
            println!("{}", serde_json::to_string_pretty(&msg).unwrap());
        }
        Format::Text => {
            println!("REJECT");
            println!("code: RejectNumericParse");
            println!("message: {}", err);
        }
    }
    process::exit(code);
}

trait DisplayResult {
    fn is_accept(&self) -> bool;
    fn to_text(&self) -> String;
}

fn decision_to_text(d: &Decision) -> String {
    match d {
        Decision::Accept => "ACCEPT".to_string(),
        Decision::Reject => "REJECT".to_string(),
        Decision::SlabBuilt => "SLAB_BUILT".to_string(),
        Decision::TerminalSuccess => "TERMINAL_SUCCESS".to_string(),
        Decision::TerminalFailure => "TERMINAL_FAILURE".to_string(),
        Decision::AbortBudget => "ABORT_BUDGET".to_string(),
        Decision::Pending => "PENDING".to_string(),
        Decision::Queued => "QUEUED".to_string(),
    }
}

impl DisplayResult for VerifyMicroResult {
    fn is_accept(&self) -> bool {
        self.decision == Decision::Accept
    }
    fn to_text(&self) -> String {
        let mut s = format!("{}\n", decision_to_text(&self.decision));
        if self.decision == Decision::Reject {
            if let Some(code) = &self.code {
                s.push_str(&format!("code: {:?}\n", code));
            }
            s.push_str(&format!("message: {}\n", self.message));
        }
        if let Some(idx) = self.step_index {
            s.push_str(&format!("step_index: {}\n", idx));
        }
        if let Some(oid) = &self.object_id {
            s.push_str(&format!("object_id: {}\n", oid));
        }
        if let Some(digest) = &self.chain_digest_next {
            s.push_str(&format!("chain_digest_next: {}\n", digest));
        }
        s
    }
}

impl DisplayResult for VerifyChainResult {
    fn is_accept(&self) -> bool {
        self.decision == Decision::Accept
    }
    fn to_text(&self) -> String {
        let mut s = format!("{}\n", decision_to_text(&self.decision));
        if self.decision == Decision::Reject {
            if let Some(code) = &self.code {
                s.push_str(&format!("code: {:?}\n", code));
            }
            s.push_str(&format!("message: {}\n", self.message));
        }
        s.push_str(&format!("steps_verified: {}\n", self.steps_verified));
        s.push_str(&format!("first_step_index: {}\n", self.first_step_index));
        s.push_str(&format!("last_step_index: {}\n", self.last_step_index));
        if let Some(digest) = &self.final_chain_digest {
            s.push_str(&format!("final_chain_digest: {}\n", digest));
        }
        if let Some(fidx) = self.failing_step_index {
            s.push_str(&format!("failing_step_index: {}\n", fidx));
        }
        s
    }
}

impl DisplayResult for BuildSlabResult {
    fn is_accept(&self) -> bool {
        self.decision == Decision::SlabBuilt
    }
    fn to_text(&self) -> String {
        let mut s = format!("{}\n", decision_to_text(&self.decision));
        if self.decision == Decision::Reject {
            if let Some(code) = &self.code {
                s.push_str(&format!("code: {:?}\n", code));
            }
            s.push_str(&format!("message: {}\n", self.message));
        } else {
            s.push_str(&format!("message: {}\n", self.message));
        }
        if let Some(rs) = self.range_start {
            s.push_str(&format!("range_start: {}\n", rs));
        }
        if let Some(re) = self.range_end {
            s.push_str(&format!("range_end: {}\n", re));
        }
        if let Some(mc) = self.micro_count {
            s.push_str(&format!("micro_count: {}\n", mc));
        }
        if let Some(root) = &self.merkle_root {
            s.push_str(&format!("merkle_root: {}\n", root));
        }
        if let Some(out) = &self.output {
            s.push_str(&format!("output: {}\n", out));
        }
        s
    }
}

impl DisplayResult for VerifySlabResult {
    fn is_accept(&self) -> bool {
        self.decision == Decision::Accept
    }
    fn to_text(&self) -> String {
        let mut s = format!("{}\n", decision_to_text(&self.decision));
        if self.decision == Decision::Reject {
            if let Some(code) = &self.code {
                s.push_str(&format!("code: {:?}\n", code));
            }
            s.push_str(&format!("message: {}\n", self.message));
        }
        s.push_str(&format!("range_start: {}\n", self.range_start));
        s.push_str(&format!("range_end: {}\n", self.range_end));
        if let Some(mc) = self.micro_count {
            s.push_str(&format!("micro_count: {}\n", mc));
        }
        if let Some(root) = &self.merkle_root {
            s.push_str(&format!("merkle_root: {}\n", root));
        }
        s
    }
}
