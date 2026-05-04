#!/usr/bin/env python3
"""
COH Verifier Bridge Module

Provides integration with the COH Python module (from coh-node Rust crate).

Usage:
    from coh_bridge import verify_receipt_coh, normalize_receipt, COH_AVAILABLE
    
    outcome, detail, path = verify_receipt_coh(receipt)
    if outcome == "ACCEPT":
        print("Receipt verified!")
"""

import sys
from typing import Optional, Tuple


# =============================================================================
# NUMERIC DOMAIN FREEZE
# =============================================================================
FLOAT_DTYPE = "float64"
MARGIN_MIN = 0.0  # Minimum margin to pass accounting law


# =============================================================================
# COH MODULE IMPORT
# =============================================================================
COH_AVAILABLE = False
COH = None

try:
    import coh
    COH = coh
    COH_AVAILABLE = True
except ImportError:
    # Try alternative import from coh-node directory
    try:
        sys.path.insert(0, "coh-node/target/release")
        sys.path.insert(0, "coh-node/target/debug")
        sys.path.insert(0, "coh-node")
        import coh
        COH = coh
        COH_AVAILABLE = True
    except ImportError:
        COH = None
        COH_AVAILABLE = False


# =============================================================================
# FALLBACK DETERMINISTIC VERIFICATION
# =============================================================================
def safe_float(v, default=0.0) -> float:
    """Convert value to float safely."""
    try:
        return float(v)
    except (TypeError, ValueError):
        return default


def _require_coh_available() -> None:
    """Fail-fast if COH module unavailable.
    
    Per Constraint 1: Python must never act as primary arbiter of state.
    If Rust module is unavailable, fail immediately rather than
    falling back to Python-only verification.
    
    Raises:
        RuntimeError: If COH module unavailable
    """
    if not COH_AVAILABLE:
        raise RuntimeError(
            "COH Rust module unavailable. Cannot verify. "
            "Python fallback disabled per Constraint 1 (verification-first architecture). "
            "Ensure Rust module is compiled and accessible."
        )


def deterministic_verify(receipt: dict) -> Tuple[str, Optional[str], str]:
    """
    DEPRECATED: Fallback verification REMOVED per Constraint 1.
    
    This function is kept only for backward compatibility with test fixtures.
    It now enforces Rust-core verification.
    
    Args:
        receipt: Micro-receipt dict
        
    Returns:
        Always delegates to Rust module (never executes here)
        
    Raises:
        RuntimeError: If COH module unavailable
    """
    # Per Constraint 1: Force Rust verification
    _require_coh_available()
    
    # Delegate to Rust - this will raise if verification fails
    try:
        COH.verify(receipt)
        return ("ACCEPT", None, "coh.verify")
    except Exception as e:
        err_type = type(e).__name__
        if err_type == "CohVerificationError" or "Verification" in err_type:
            return ("REJECT_MARGIN", str(e), "coh.reject")
        elif err_type == "CohMalformedError" or "Malformed" in err_type:
            return ("MALFORMED", str(e), "coh.malformed")
        else:
            return ("MALFORMED", f"[{err_type}] {str(e)}", "coh.unknown")


# =============================================================================
# COH VERIFICATION
# =============================================================================
def verify_receipt_coh(receipt: dict) -> Tuple[str, Optional[str], str]:
    """
    Verify a micro-receipt with COH module.
    
    Per Constraint 1: Must use Rust core - never fallback to Python.
    
    Args:
        receipt: Micro-receipt dict
    
    Returns:
        Tuple of (outcome, detail, verification_path)
        
    Raises:
        RuntimeError: If COH module unavailable
    """
    # Always require Rust - fail fast if unavailable
    _require_coh_available()
    
    try:
        COH.verify(receipt)
        return ("ACCEPT", None, "coh.verify")
    except Exception as e:
        # Map PyO3 exception names to internal outcomes
        err_type = type(e).__name__
        if err_type == "CohVerificationError" or "Verification" in err_type:
            return ("REJECT_MARGIN", str(e), "coh.reject")
        elif err_type == "CohMalformedError" or "Malformed" in err_type:
            return ("MALFORMED", str(e), "coh.malformed")
        else:
            return ("MALFORMED", f"[{err_type}] {str(e)}", "coh.unknown")


def normalize_receipt(receipt: dict) -> Optional[dict]:
    """
    Normalize a receipt to compute chain digest.
    
    Args:
        receipt: Micro-receipt dict
    
    Returns:
        Receipt with chain_digest_next populated, or unchanged if COH unavailable
    """
    if not COH_AVAILABLE:
        return receipt
    
    try:
        norm = COH.normalize(receipt)
        if hasattr(norm, "hash"):
            receipt["chain_digest_next"] = norm.hash
    except Exception as e:
        print(f"Normalize failed: {e}", file=sys.stderr)
    
    return receipt


def verify_chain_coh(chain_receipts: list[dict]) -> Tuple[str, Optional[str], str]:
    """
    Verify a chain of receipts.
    
    Args:
        chain_receipts: List of micro-receipts
    
    Returns:
        Tuple of (outcome, detail, path)
    """
    if not COH_AVAILABLE:
        # Deterministic chain validation
        prev_digest = "0" * 64
        for i, receipt in enumerate(chain_receipts):
            outcome, detail, path = verify_receipt_coh(receipt)
            if outcome != "ACCEPT":
                return (outcome, f"step {i}: {detail}", "deterministic.chain")
            
            # Check chain linkage
            if receipt.get("chain_digest_prev") != prev_digest:
                return ("REJECT_MARGIN", f"step {i} broken link", "deterministic.link")
            
            prev_digest = receipt.get("chain_digest_next", "0" * 64)
        
        return ("ACCEPT", f"{len(chain_receipts)} steps", "deterministic.chain_ok")
    
    try:
        import json
        chain_json = "\n".join(json.dumps(r) for r in chain_receipts)
        result = COH.verify_chain(chain_json)
        return ("ACCEPT", result, "coh.chain")
    except Exception as e:
        return ("REJECT_MARGIN", str(e), "coh.chain_fail")


def verify_slab_coh(slab: dict) -> Tuple[str, Optional[str], str]:
    """
    Verify a slab receipt.
    
    Args:
        slab: Slab receipt dict
    
    Returns:
        Tuple of (outcome, detail, path)
    """
    if not COH_AVAILABLE:
        return ("REJECT_MARGIN", "slab verify requires COH", "deterministic.slab_unavailable")
    
    try:
        result = COH.verify_slab(slab)
        return ("ACCEPT", result, "coh.slab")
    except Exception as e:
        return ("REJECT_MARGIN", str(e), "coh.slab_fail")


# =============================================================================
# ERROR CLASSES
# =============================================================================
class CohVerificationError(Exception):
    """Raised when COH verification fails."""
    pass


class CohMalformedError(Exception):
    """Raised when receipt is malformed."""
    pass


# =============================================================================
# SELF-TEST
# =============================================================================
if __name__ == "__main__":
    print("COH Verifier Bridge Module")
    print("=" * 40)
    
    print(f"COH module available: {COH_AVAILABLE}")
    
    # Test receipt structure
    test_receipt = {
        "schema_id": "coh.receipt.micro.v1",
        "version": "1.0.0",
        "object_id": "test.1",
        "step_index": 0,
        "metrics": {
            "v_pre": "100",
            "v_post": "70",
            "spend": "20",
            "defect": "5",
        },
    }
    
    print(f"\nTest receipt: {test_receipt['object_id']}")
    outcome, detail, path = verify_receipt_coh(test_receipt)
    print(f"  Outcome: {outcome}")
    print(f"  Detail: {detail}")
    print(f"  Path: {path}")