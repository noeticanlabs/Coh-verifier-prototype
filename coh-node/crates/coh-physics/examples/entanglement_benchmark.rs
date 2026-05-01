// fixture_only: allow_mock
use coh_core::atom::CohAtom;
use coh_core::types::{Hash32, DomainId, Signature, RvStatus};
use coh_core::entanglement::{EntangledCohAtom, EntanglementReject, CouplingWitnessKind};
use num_rational::Rational64;

fn main() {
    let domain = DomainId(Hash32([1; 32]));
    let policy_hash = Hash32([0x77; 32]);
    println!("Starting Coh-wedge Entanglement v2.4 Benchmark\n");

    // --- SETUP ATOMS ---
    // Atom 1: Locally Inadmissible
    let mut atom1 = CohAtom::identity(Hash32([0x11; 32]), Rational64::from_integer(500), domain);
    atom1.policy_hash = policy_hash;
    atom1.atom_id = Hash32([0xA1; 32]);
    let bit1 = &mut atom1.bits[0];
    bit1.bit_id = Hash32([0xB1; 32]);
    bit1.action_hash = Hash32([0xC1; 32]);
    bit1.signature = Signature(vec![1; 64]);
    bit1.rv_status = RvStatus::Accept;
    bit1.spend = Rational64::from_integer(1000);
    bit1.receipt_hash = bit1.canonical_hash();
    atom1.cumulative_spend = Rational64::from_integer(1000);
    atom1.atom_hash = atom1.canonical_hash();

    // Atom 2: Locally Inadmissible
    let mut atom2 = CohAtom::identity(Hash32([0x22; 32]), Rational64::from_integer(400), domain);
    atom2.policy_hash = policy_hash;
    atom2.atom_id = Hash32([0xA2; 32]);
    let bit2 = &mut atom2.bits[0];
    bit2.bit_id = Hash32([0xB2; 32]);
    bit2.action_hash = Hash32([0xC2; 32]);
    bit2.signature = Signature(vec![1; 64]);
    bit2.rv_status = RvStatus::Accept;
    bit2.spend = Rational64::from_integer(800);
    bit2.receipt_hash = bit2.canonical_hash();
    atom2.cumulative_spend = Rational64::from_integer(800);
    atom2.atom_hash = atom2.canonical_hash();

    // --- CASE 1: Joint Pass (Locally failing) ---
    println!("Case 1: A local fail, B local fail, joint pass");
    let shared_authority = Rational64::from_integer(2000);
    let shared_authority_cap = Rational64::from_integer(2000);
    let shared_defect = Rational64::from_integer(0);
    let shared_delta_hat = Rational64::from_integer(0);
    let monogamy_scope = Hash32([0xEE; 32]);
    let coupling_witness = Hash32([0xAA; 32]);

    let entanglement = EntangledCohAtom::new(
        vec![atom1.clone(), atom2.clone()],
        shared_defect,
        shared_delta_hat,
        shared_authority,
        shared_authority_cap,
        domain,
        policy_hash,
        monogamy_scope,
        CouplingWitnessKind::HeuristicCorrelation,
        coupling_witness,
    );

    println!("  Joint Margin: {}", entanglement.joint_margin);
    match entanglement.verify(&[], false) {
        Ok(_) => println!("  Result: ACCEPT ✅"),
        Err(e) => panic!("  Result: FAILED ❌ ({:?})", e),
    }

    // --- CASE 2: Alone execution rejection ---
    println!("\nCase 2: A tries to execute alone under joint-only validity");
    println!("  Atom 1 executable? {}", atom1.executable());
    if !atom1.executable() {
        println!("  Result: REJECT ✅");
    } else {
        panic!("  Result: FAILED ❌ (Atom 1 should be locally inadmissible)");
    }

    // --- CASE 3: Monogamy scope reused ---
    println!("\nCase 3: same monogamy_scope reused (global key collision)");
    let key = entanglement.monogamy_key();
    match entanglement.verify(&[key], false) {
        Err(EntanglementReject::MonogamyViolation) => println!("  Result: REJECT ✅"),
        _ => panic!("  Result: FAILED ❌ (Reuse not caught)"),
    }

    // --- CASE 4: Fixture witness in production context ---
    println!("\nCase 4: fixture witness in production context");
    let mut entanglement_fixture = entanglement.clone();
    entanglement_fixture.witness_kind = CouplingWitnessKind::FixtureOnly;
    match entanglement_fixture.verify(&[], true) {
        Err(EntanglementReject::WitnessInvalid) => println!("  Result: REJECT ✅"),
        _ => panic!("  Result: FAILED ❌ (Fixture witness allowed in production)"),
    }

    println!("\nCOH ENTANGLEMENT V2.4 BENCHMARK COMPLETE.");
}
