# Coherent Validator Core

A deterministic state transition validator with cryptographic tamper detection for blockchain execution verification.

## Overview

This crate provides the core verification engine for state transition receipts. It validates:

- **Micro-receipts**: Individual state transition records
- **Chains**: Linked sequences of receipts with continuity enforcement  
- **Slabs**: Compressed macro receipts with Merkle root integrity

## Features

- **Deterministic verification**: Same input always produces same output
- **Cryptographic tamper detection**: SHA256 digest verification
- **Arithmetic safety**: Checked math prevents overflow attacks
- **Policy enforcement**: v_post + spend <= v_pre + defect
- **Continuity enforcement**: Step order, state linkage, digest linkage

## Installation

```toml
[dependencies]
coh-core = "0.1.0"
```

## Quick Start

```rust
use coh_core::{verify_micro, verify_chain, build_slab, verify_slab};
use coh_core::types::MicroReceiptWire;
use serde_json::from_str;

// Verify a single receipt
let json = r#"{"schema_id":"coh.receipt.micro.v1",...}"#;
let receipt: MicroReceiptWire = from_str(json).unwrap();
let result = verify_micro(receipt);

match result.decision {
    Decision::Accept => println!("Verified!"),
    Decision::Reject => println!("Rejected: {:?}", result.code),
}
```

## API Reference

### verify_micro

Validates a single micro-receipt:

```rust
pub fn verify_micro(wire: MicroReceiptWire) -> VerifyMicroResult
```

Checks:
- Schema ID and version
- Canon profile hash
- Policy arithmetic (no overflow, inequality holds)
- Cryptographic digest

### verify_chain

Validates a sequence of receipts:

```rust
pub fn verify_chain(receipts: Vec<MicroReceiptWire>) -> VerifyChainResult
```

Checks:
- Each receipt individually
- Step index continuity (strictly +1)
- State hash linkage
- Chain digest linkage

### build_slab

Constructs a macro receipt from a chain:

```rust
pub fn build_slab(receipts: Vec<MicroReceiptWire>) -> BuildSlabResult
```

Produces:
- Range (first_step, last_step)
- Micro count
- Merkle root
- Aggregate accounting

### verify_slab

Validates a standalone slab:

```rust
pub fn verify_slab(wire: SlabReceiptWire) -> VerifySlabResult
```

Checks:
- Range and count consistency
- Merkle root integrity
- Macro policy inequality
- Summary arithmetic

## Types

### Decision

```rust
pub enum Decision {
    Accept,
    Reject,
    SlabBuilt,
}
```

### RejectCode

```rust
pub enum RejectCode {
    RejectSchema,
    RejectCanonProfile,
    RejectChainDigest,
    RejectStateHashLink,
    RejectNumericParse,
    RejectOverflow,
    RejectPolicyViolation,
    RejectSlabSummary,
    RejectSlabMerkle,
}
```

## Examples

See `examples/` directory for JSON format examples:
- `micro_valid.json` - Valid micro receipt
- `chain_valid.jsonl` - Valid chain (JSONL)
- `slab_valid.json` - Valid slab

## Testing

```bash
cargo test
```

Run specific test suites:
```bash
cargo test --test test_verify_micro
cargo test --test test_verify_chain
cargo test --test test_verify_slab
```

## Performance

The validator uses:
- Stack-allocated types where possible
-SHA256 for digest computation
- Checked arithmetic (no panic on overflow)

Typical throughput: ~10,000 receipts/second on modern hardware.

## License

MIT