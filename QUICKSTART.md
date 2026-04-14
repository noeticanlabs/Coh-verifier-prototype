# End-to-End Verification Walkthrough

This guide walks through the complete verification pipeline from raw AI action to verified receipt.

---

## Quick Start (5 minutes)

### 1. Verify a Single Receipt

```bash
# From project root
cd coh-node
cargo run --release --package coh-cli -- verify-micro examples/micro_valid.json
```

**Expected Output**:
```
ACCEPT
```

### 2. Verify a Chain

```bash
cargo run --release --package coh-cli -- verify-chain examples/chain_valid.jsonl
```

**Expected Output**:
```
ACCEPT
Chain verified: 10 receipts
```

### 3. Build a Slab (Aggregate)

```bash
cargo run --release --package coh-cli -- build-slab examples/chain_valid.jsonl --out slab_output.json
```

### 4. Verify the Slab

```bash
cargo run --release --package coh-cli -- verify-slab slab_output.json
```

---

## Complete Pipeline Diagram

```
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│   AI System         │     │   verify_micro      │     │   verify_chain       │
│   Raw Receipt       │────▶│   (single receipt)   │────▶│   (sequence)         │
│   (JSON)            │     │                     │     │                     │
│                     │     │ Schema check        │     │ Index continuity    │
│ Example:           │     │ Policy check        │     │ State linkage       │
│ {                  │     │ Digest verify       │     │ Digest linkage      │
│   "step_index": 0, │     │                     │     │                     │
│   "v_pre": "100",  │     └─────────────────────┘     └──────────┬──────────┘
│   "v_post": "95",  │                                        │
│   "spend": "5"     │                                        ▼
│ }                  │                            ┌─────────────────────┐
└─────────────────────┘                            │   build_slab        │
                                                     │   (aggregate)       │
                                                     │                     │
                                                     │ Merkle root         │
                                                     │ Summary compute     │
                                                     └──────────┬──────────┘
                                                                │
                                                                ▼
                                                     ┌─────────────────────┐
                                                     │   verify_slab       │
                                                     │   (macro receipt)   │
                                                     │                     │
                                                     │ Range check         │
                                                     │ Merkle verify       │
                                                     │ Macro policy        │
                                                     └─────────────────────┘
```

---

## Input Formats

### Micro Receipt (JSON)

```json
{
  "schema_id": "coh.receipt.micro.v1",
  "step_index": 0,
  "state_pre": "abc123",
  "state_post": "def456",
  "canon_profile_hash": "sha256:...",
  "chain_digest_prev": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "metrics": {
    "v_pre": "1000",
    "v_post": "950",
    "spend": "30",
    "defect": "5",
    "authority": "10"
  }
}
```

### Chain (JSONL - one JSON per line)

```jsonl
{"step_index": 0, ...}
{"step_index": 1, ...}
{"step_index": 2, ...}
```

### Slab (JSON)

```json
{
  "schema_id": "coh.receipt.slab.v1",
  "first_step": 0,
  "last_step": 9,
  "micro_count": 10,
  "merkle_root": "sha256:...",
  "summary": {
    "total_spend": "300",
    "total_defect": "50"
  }
}
```

---

## Reject Codes (Troubleshooting)

| Code | Meaning | Fix |
|------|---------|-----|
| `RejectSchema` | Invalid schema ID/version | Check schema_id matches expected format |
| `RejectPolicyViolation` | Accounting law violated | Ensure `v_post + spend ≤ v_pre + defect + authority` |
| `RejectStateHashLink` | State continuity broken | Check `state_post` of step N matches `state_pre` of step N+1 |
| `RejectChainDigest` | Chain integrity broken | Verify chain digest linkage |
| `RejectOverflow` | Arithmetic overflow | Use checked math (u128 max) |

---

## Testing with Examples

The project includes test vectors:

```bash
# Valid examples
ls examples/micro_valid.json
ls examples/chain_valid.jsonl
ls examples/slab_valid.json

# Invalid examples (should all REJECT)
ls examples/micro_invalid_*.json
ls examples/chain_invalid_*.jsonl
ls examples/slab_invalid_*.json

# AI demo examples
ls examples/ai_demo/*.json
```

Run all test vectors:

```bash
cd coh-node
cargo test --test integration
```

---

## Python Integration

```python
import subprocess
import json

# Verify a receipt via CLI
result = subprocess.run(
    ["coh-validator", "verify-micro", "receipt.json"],
    capture_output=True,
    text=True
)

if "ACCEPT" in result.stdout:
    print("Receipt verified!")
else:
    print(f"Rejected: {result.stdout}")
```

---

## Next Steps

- [ ] Try the [Dashboard](coh-dashboard/README.md) for visual inspection
- [ ] Read [SECURITY_MODEL.md](SECURITY_MODEL.md) for threat model details
- [ ] Explore [Lean T-Stack](coh-t-stack/) for formal proofs