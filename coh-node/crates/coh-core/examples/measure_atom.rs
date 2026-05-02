use coh_core::atom::CohAtom;
use coh_core::cohbit::CohBit;
use coh_core::types::Hash32;
use num_rational::Rational64;

fn main() {
    println!("Size of CohAtom: {} bytes", std::mem::size_of::<CohAtom>());
    println!("Alignment of CohAtom: {} bytes", std::mem::align_of::<CohAtom>());
    println!("Size of CohBit: {} bytes", std::mem::size_of::<CohBit>());
    
    println!("--- Field breakdown ---");
    println!("version: u16 -> {}", std::mem::size_of::<u16>());
    println!("kind: AtomKind -> {}", std::mem::size_of::<coh_core::atom::AtomKind>());
    println!("domain: DomainId -> {}", std::mem::size_of::<coh_core::types::DomainId>());
    println!("atom_id: Hash32 -> {}", std::mem::size_of::<Hash32>());
    println!("bits: Vec<CohBit> -> {}", std::mem::size_of::<Vec<CohBit>>());
    println!("cumulative_spend: Rational64 -> {}", std::mem::size_of::<Rational64>());
    println!("signature: Signature -> {}", std::mem::size_of::<coh_core::types::Signature>());
    println!("compression_certificate: Option<Hash32> -> {}", std::mem::size_of::<Option<Hash32>>());
}
