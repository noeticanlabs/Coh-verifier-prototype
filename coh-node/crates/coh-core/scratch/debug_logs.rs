use coh_core::external::logs::{ingest_cicd_jsonl, run_logs_validation};
use std::path::Path;

fn main() {
    let path = "coh-node/examples/logs/cicd_jobs.jsonl";
    println!("Testing ingestion of: {}", path);
    
    match ingest_cicd_jsonl(path) {
        Ok(receipts) => {
            println!("Ingested {} receipts.", receipts.len());
            let report = run_logs_validation(receipts.clone());
            println!("Report: {:?}", report);
            
            // Debug the first failure if any
            if report.rejected_valid > 0 {
                let r = &receipts[0];
                let res = coh_core::verify_micro::verify_micro(r.clone());
                println!("First Step Reject: {:?} - {}", res.code, res.message);
            }
        }
        Err(e) => println!("Ingestion failed: {}", e),
    }
}
