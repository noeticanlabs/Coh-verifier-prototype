use clap::{Parser, Subcommand};
use coh_genesis::ctrl::CtrlLoop;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "coh-ctrl")]
#[command(about = "Certified Theorem Repair Loop (CTRL) CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Repair a theorem with a list of candidate tactics
    Repair {
        /// Path to the Lean project
        #[arg(short, long)]
        project: PathBuf,

        /// Name of the theorem to repair
        #[arg(short, long)]
        theorem: String,

        /// List of tactics to try (comma-separated)
        #[arg(short, long)]
        tactics: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Repair { project, theorem, tactics } => {
            println!("Starting CTRL for theorem: {}", theorem);
            let mut ctrl = CtrlLoop::new(project.clone())?;
            
            let candidates: Vec<&str> = tactics.split(',').map(|s| s.trim()).collect();
            let result = ctrl.repair_theorem(theorem, candidates)?;

            if result.success {
                println!("SUCCESS: Theorem '{}' repaired using tactic: {}", result.theorem, result.tactic);
                println!("Proof Hash: {}", result.proof_hash);
            } else {
                println!("FAILURE: Could not repair theorem '{}' with provided tactics.", theorem);
            }
        }
    }

    Ok(())
}
