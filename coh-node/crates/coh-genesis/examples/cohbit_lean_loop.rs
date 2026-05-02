// fixture_only: allow_mock
use coh_core::vm::CohVM;
use coh_core::types::{Hash32, DomainId};
use coh_core::atom::CohGovernor;
use coh_core::spinor::CohSpinor;
use coh_genesis::vm_runtime_lean::LeanCohBitRuntime;
use num_rational::Rational64;
use std::path::PathBuf;

fn main() {
    let domain = DomainId(Hash32([1; 32]));
    let project_path = PathBuf::from("../coh-t-stack");
    
    // 1. Initialize the Certified Lean Runtime
    let mut runtime = LeanCohBitRuntime::new(domain, project_path);
    
    // 2. Set up the VM for a proof search
    let initial_state = Hash32([0; 32]);
    let governor = CohGovernor::default();
    let spinor = CohSpinor::default();
    let spin_ctx = coh_core::spinor::SpinContext {
        k1_amplification: Rational64::new(1, 10),
        k2_decay: Rational64::new(1, 100),
        tau_drift: Rational64::new(1, 1000),
    };
    let verifier_ctx = coh_core::vm::VerifierContext {
        policy_hash: Hash32([7; 32]),
        verifier_id: Hash32([8; 32]),
    };
    
    let mut vm = CohVM::new(initial_state, governor, spinor, spin_ctx, verifier_ctx);
    
    println!("Starting Certified Lean Loop via CohBit System\n");
    
    // 3. Run the loop for 3 proof steps
    for i in 0..3 {
        println!("Step {}: Current proof state: {:?}", i, vm.state);
        
        match vm.step(&mut runtime) {
            Ok(bit) => {
                println!("  Transition ACCEPTED ✅");
                println!("  Proof Certificate: {:?}", bit.certificate_hash);
                println!("  New proof state: {:?}\n", vm.state);
            },
            Err(e) => {
                println!("  Transition REJECTED ❌: {}\n", e);
                break;
            }
        }
    }
    
    // 4. Finalize the Proof Atom
    match vm.finalize_atom(&mut runtime) {
        Ok(atom) => {
            println!("Certified Proof Atom finalized! Bits: {}", atom.bits.len());
            println!("Atom Hash: {:?}", atom.atom_hash);
        },
        Err(e) => println!("Finalization failed: {:?}", e),
    }
    
    println!("\nCertified Lean Loop completed.");
}
