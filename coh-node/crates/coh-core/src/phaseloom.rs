//! PhaseLoom Ecology Verification - Simplified for core kernel integration
//!
//! This module provides the core thermodynamic laws for memory and budget
//! that must be enforced during micro-step verification.

use crate::reject::RejectCode;

/// Provenance authority rank
pub fn provenance_authority(prov: &str) -> u8 {
    match prov {
        "EXT" => 4,
        "DER" => 3,
        "REP" => 2,
        "SIM" => 1,
        _ => 0,
    }
}

/// [PHASELOOM ECOLOGY: Lawful Recall]
/// Calculates the read cost for a memory access
pub fn calculate_read_cost(
    current_tau: u64,
    record_tau: u64,
    provenance: &str,
) -> u128 {
    let dt = current_tau.saturating_sub(record_tau);
    let prov_cost = (4 - provenance_authority(provenance)) as u128 * 10;
    
    // cost = dt * alpha_tau + prov_dist * alpha_p
    (dt as u128 * 2) + prov_cost
}

/// [PHASELOOM ECOLOGY: Anchor Firewall]
/// Validates that a memory transition does not violate the provenance lattice
pub fn validate_anchor_transition(
    old_prov: &str,
    new_prov: &str,
) -> Result<(), RejectCode> {
    let old_auth = provenance_authority(old_prov);
    let new_auth = provenance_authority(new_prov);
    
    if new_auth < old_auth {
        Err(RejectCode::PhaseLoomEpistemicViolation)
    } else {
        Ok(())
    }
}
