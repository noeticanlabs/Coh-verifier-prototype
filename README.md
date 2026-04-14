# Coh Safety Wedge

**Deterministic AI Verification Kernel & Formal T-Stack Ledger**

[![CI](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml/badge.svg)](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Lean: v4.16.0](https://img.shields.io/badge/lean-v4.16.0-blue.svg)](https://leanprover.github.io/)
[![License: Proprietary](https://img.shields.io/badge/license-Proprietary-red.svg)](LICENSE)

`ai-safety` `determinism` `rust` `lean4` `verification-kernel` `integrity-audit`

## Overview

The **Coh Safety Wedge** is the high-integrity core of the Coh Network. It provides a dual-layer security guarantee:
1. **Rust Verification Kernel**: A high-performance, deterministic engine for auditing AI receipt chains and state transitions.
2. **Lean T-Stack Ledger**: A machine-verified formal foundation proving the categorical and physical invariants of the safety contract.

## Quick Start

New to Coh? Try the [QUICKSTART.md](QUICKSTART.md) for a 5-minute end-to-end walkthrough.

## Project Structure

- **`coh-node/`**: The production Rust workspace.
  - `crates/coh-core/`: Core verification logic (JCS, SHA-256, Accounting Law).
  - `crates/coh-cli/`: `coh-validator` CLI for manual and automated auditing.
  - `crates/coh-python/`: High-level bindings for AI workflow integration.
  - `crates/coh-sidecar/`: Axum-based REST API for remote verification.
- **`coh-t-stack/`**: The Formal T-Stack Ledger (Lean 4).
  - `Coh/Ledger/`: Verified theorems (T1: Strict Coh ? Category).
- **`coh-dashboard/`**: The Integrity Inspector (React/Vite).
  - Visual timeline and audit inspector for AI receipt chains.

## Core Concepts

### T-Stack (The Federated Ledger)

The **T-Stack** is a layered proof system that verifies AI behavior at different granularities:

| Layer | What it proves | Analogy |
|-------|----------------|---------|
| **T1** | Individual actions are well-formed (categorical structure) | Grammar check |
| **T2** | Actions respect operational slack (oplax bridge) | Tolerance bounds |
| **T3** | Multiple actions aggregate correctly (macro-slab) | Running total |
| **T4** | Errors are visible to auditors (visibility) | Audit trail |
| **T5** | System selects minimal valid paths (dirac selection) | Optimization |

Each T-layer catches different failure modes, forming a **defense-in-depth** verification stack.

### Accounting Law

The fundamental **budget constraint** enforced on every AI action:

```
v_post + spend ≤ v_pre + defect + authority
```

| Term | Meaning | Example |
|------|---------|---------|
| `v_pre` | Value before action | 1000 tokens |
| `v_post` | Value after action | 950 tokens |
| `spend` | Resources consumed | 30 tokens |
| `defect` | Known losses/bugs | 5 tokens |
| `authority` | External additions | 10 tokens |

**Decision**: ACCEPT if inequality holds, REJECT otherwise.

This prevents AI systems from "spending" resources they don't have — a fundamental integrity invariant.

### Receipt Chain

A **tamper-evident log** of all AI actions, where each entry:
1. References the previous entry's hash (chain integrity)
2. Contains a canonical (deterministic) representation (JCS)
3. Includes a Merkle root for efficient auditing

```
Receipt 0 → Hash0
Receipt 1 → Hash1 = SHA256(Hash0 | Canonical(Receipt1))
Receipt 2 → Hash2 = SHA256(Hash1 | Canonical(Receipt2))
```

Altering a past receipt breaks the chain — providing **non-repudiation**.

## Formal Foundations

The system is anchored by the **T-Stack Federated Ledger**. Every foundational claim is machine-verified using Lean 4 to ensure total mathematical soundness. See [FORMAL_FOUNDATION.md](FORMAL_FOUNDATION.md) for the complete theorem mapping.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     UNTRUSTED ZONE                          │
│  AI System Output → Raw Receipts                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      WEDGE (Rust Kernel)                    │
│  verify_micro → verify_chain → verify_slab                 │
│  [Decision: ACCEPT or REJECT with RejectCode]              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    TRUSTED ZONE                             │
│  Verified Receipt Chain → Dashboard / Audit Trail          │
└─────────────────────────────────────────────────────────────┘
```

## Security Model

For detailed threat model, trust boundaries, and security assumptions, see [SECURITY_MODEL.md](SECURITY_MODEL.md).

## Development

### Prerequisites
- Rust stable
- Lean 4 (Elan)
- Node.js 20+

### Building the Kernel
```bash
cd coh-node
cargo build --release
```

### Building the Ledger
```bash
cd coh-t-stack
lake build
```

### Running the Dashboard
```bash
cd coh-dashboard
npm install
npm run dev
```

### Docker Quick Start

For containerized execution without installing Rust:

```bash
# Build and run verifier
docker build -f coh-node/Dockerfile -t coh-validator .
docker run -v ./receipts:/data coh-validator verify-micro /data/input.json

# Or use Docker Compose
docker-compose up --profile interactive
```

## License
Proprietary - Noetican Labs. All rights reserved.
