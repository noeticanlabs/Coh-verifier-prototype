# Security Model

**Document Type**: Threat Model & Security Assumptions  
**Last Updated**: 2026-04-14

---

## 1. What We're Defending Against

The Coh Safety Wedge provides defense against the following threat categories:

| Threat | Description | Defense Mechanism |
|--------|-------------|-------------------|
| **Tampering** | Modification of historical receipts | SHA-256 chain hashing, Merkle root verification |
| **Resource Exhaustion** | Arithmetic overflow/underflow in AI accounting | Checked math (`checked_add/sub/mul`), bounded loops |
| **Policy Bypass** | AI exceeding allocated resources/authority | T2 oplax bridge, invariant enforcement |
| **Equivocation** | Different representations of same action | Canonical JSON (JCS), deterministic verification |
| **Replay Attacks** | Re-submission of valid receipts | Index continuity checks, state link verification |
| **Chain Break** | Gaps or reorderings in receipt sequence | Strict step index (+1) enforcement |

---

## 2. Trust Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                     UNTRUSTED ZONE                          │
│                                                              │
│  AI System Output                                           │
│  Raw Receipts (JSON/JSONL)                                  │
│  External Inputs                                            │
│                                                              │
│  ASSUMPTION: These inputs are potentially malicious        │
│              or malformed                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼  INPUT VALIDATION STARTS
┌─────────────────────────────────────────────────────────────┐
│                      WEDGE (Rust Kernel)                    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ verify_micro.rs  — Single receipt validation        │   │
│  │   • Schema validation                                 │   │
│  │   • Policy inequality check                           │   │
│  │   • Digest verification                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                              │                              │
│                              ▼                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ verify_chain.rs — Chain continuity validation       │   │
│  │   • Step index continuity                            │   │
│  │   • State hash linkage                               │   │
│  │   • Chain digest linkage                             │   │
│  └─────────────────────────────────────────────────────┘   │
│                              │                              │
│                              ▼                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ verify_slab.rs   — Macro receipt validation         │   │
│  │   • Merkle root verification                         │   │
│  │   • Aggregate accounting                             │   │
│  │   • Range consistency                                │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
│  OUTPUT: Decision (ACCEPT/REJECT) + RejectCode             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼  VERIFIED OUTPUT
┌─────────────────────────────────────────────────────────────┐
│                    TRUSTED ZONE                             │
│                                                              │
│  Verified Receipt Chain                                     │
│  Audit Trail / Dashboard Visualization                      │
│  Downstream Systems (if ACCEPT)                            │
│                                                              │
│  ASSUMPTION: These systems are trusted to handle           │
│              verified data correctly                       │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Security Assumptions

### 3.1 Cryptographic Assumptions

| Assumption | Description |
|------------|-------------|
| **SHA-256 Collision Resistance** | We assume SHA-256 is collision-resistant for chain hashing and Merkle tree construction. |
| **Deterministic Hashing** | Hashing is deterministic given the same input (guaranteed by JCS canonicalization). |

### 3.2 Runtime Assumptions

| Assumption | Description |
|------------|-------------|
| **Honest Verifier** | The Rust kernel runs in a trusted environment. An attacker with kernel access can bypass verification. |
| **No Side Channels** | Timing attacks, power analysis, and other side-channel attacks are out of scope. |
| **Well-Formed JSON** | Input parsing assumes JSON is well-formed (schema validation happens after parsing). |

### 3.3 AI System Assumptions

| Assumption | Description |
|------------|-------------|
| **Action Capture** | We assume all AI actions are captured as receipts. Side-effects without receipts are not verified. |
| **Non-Repudiation** | The AI system cannot deny generating a receipt that passes verification (due to chain linkage). |

---

## 4. What We Don't Defend Against

The following are explicitly out of scope:

| Threat | Reason |
|--------|--------|
| **Physical Security** | Host machine physical security is not addressed |
| **Insider Threats** | Malicious operator with kernel access |
| **Denial of Service** | Network-level DoS attacks |
| **Prompt Injection / Jailbreak** | We verify actions, not AI intent or prompt safety |
| **Data Privacy** | Privacy of data in receipts is not addressed |
| **Key Management** | Signing key management is out of scope (not yet implemented) |

---

## 5. Failure Modes & Responses

| Failure Mode | Symptom | Response |
|--------------|---------|----------|
| **Malformed Input** | `RejectCode::Schema` | Reject, no recovery |
| **Chain Break** | `RejectCode::StateHashLink` or `RejectCode::ChainDigest` | Halt, require rescan |
| **Overflow Attempt** | `RejectCode::Overflow` | Checked math prevents, halt if detected |
| **Policy Violation** | `RejectCode::PolicyViolation` | Log violation, reject |
| **Merkle Mismatch** | `RejectCode::SlabMerkle` | Recompute slab, reject on mismatch |
| **Invalid Summary** | `RejectCode::SlabSummary` | Reject macro receipt |

---

## 6. Reject Code Taxonomy

| Code | Category | Description |
|------|----------|-------------|
| `RejectSchema` | Input | Schema ID or version mismatch |
| `RejectCanonProfile` | Canonicalization | Canon profile hash mismatch |
| `RejectChainDigest` | Chain Integrity | Chain digest does not match previous entry |
| `RejectStateHashLink` | State Continuity | State hash linkage broken |
| `RejectNumericParse` | Arithmetic | Numeric field parsing failure |
| `RejectOverflow` | Arithmetic | Arithmetic overflow/underflow detected |
| `RejectPolicyViolation` | Policy | Accounting law violated (v_post + spend > v_pre + defect + authority) |
| `RejectSlabSummary` | Slab | Slab summary arithmetic error |
| `RejectSlabMerkle` | Slab | Merkle root verification failed |

---

## 7. Future Security Improvements

### V2 Roadmap

1. **Multi-party Signatures**: Sidecar will support signature aggregation for distributed verification
2. **Encrypted Receipts**: Support for encrypted payload with authorized decryption keys
3. **Audit Log immutability**: Blockchain-based audit trail for verified receipts

---

## 8. Contact

For security issues or questions, contact: [SECURITY.md placeholder]

---

*This document will be updated as the threat model evolves.*