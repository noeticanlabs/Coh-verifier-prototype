# Changelog

All notable changes to the Coh Validator are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Version numbers follow [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added

**CI Infrastructure**
- Demo test script (`tests/demo_test.sh`) for automated regression testing in CI
- Dashboard UI test suite using Vitest + React Testing Library
- Tests verify real demo data from `public/demo/` (not mocked)

**Dashboard**
- Scenario-driven fixture loading
- Live sidecar verification option
- Real-time chain break derivation
- SlabCheck validation visualization

### Fixed

**CLI Exit Codes**
- Documented exit code 4 (SOURCE) removed - implementation uses 0-3 only
- Docs now match implementation: 0=accept, 1=reject, 2=malformed, 3=error

**Documentation**
- Error/Reject contract created (plans/ERROR_REJECT_CONTRACT.md)
- Interface behavior matrix (plans/INTERFACE_BEHAVIOR_MATRIX.md)
- Sidecar API documentation (plans/SIDECAR_API.md)
- Python bindings guide (plans/PYTHON_BINDINGS.md)
- Receipt schema specification (plans/RECEIPT_SCHEMA_SPEC.md)
- Dashboard architecture doc (plans/DASHBOARD_ARCHITECTURE.md)

---

## [0.1.0] — 2026-04-10

### Added

**Core Protocol (coh-core)**

- `verify_micro` — 6-step deterministic single-receipt verifier (schema, version, object_id, canon profile, policy inequality, chain digest)
- `verify_chain` — JSONL chain verifier with step_index continuity, state_hash linkage, and chain_digest linkage checks
- `build_slab` — chain aggregator producing a Merkle-rooted slab receipt with checked arithmetic totals
- `verify_slab` — standalone slab macro-inequality verifier
- `verify_slab_with_leaves` — full slab verification including Merkle root recomputation

**Data Model**

- 4-layer architecture: Wire (decimal strings) → Runtime (u128) → Prehash (alphabetized) → Result (typed decision)
- `MicroReceiptWire`, `SlabReceiptWire` with `#[serde(deny_unknown_fields)]`
- `Hash32` type with length-validated hex parsing
- `RejectCode` enum (9 codes) covering all rejection categories
- Non-circular digest design: `chain_digest_next` structurally excluded from prehash view

**Cryptography**

- SHA-256 chain digest with domain tag `COH_V1_CHAIN`
- SHA-256 Merkle inner nodes with domain tag `COH_V1_MERKLE`
- Odd-leaf Merkle tree with self-duplication

**Arithmetic Safety**

- `CheckedMath` trait with `safe_add`, `safe_sub`, `safe_mul` — no raw arithmetic operators in verifier logic
- All numeric fields stored as `u128`; no floating-point anywhere

**CLI (coh-validator)**

- Commands: `verify-micro`, `verify-chain`, `build-slab`, `verify-slab`
- Exit code contract: 0 ACCEPT, 1 REJECT, 2 MALFORMED, 3 ERROR, 4 SOURCE
- Output formats: `--format text` (default), `--format json`

**Testing**

- 11 fixture files (micro, chain, slab — valid and invalid)
- 7 CLI exit-code integration tests
- Fixture oracle sweep test (all fixtures verified against expected decisions)
- Digest stability golden test
- Canonicalization byte-level test

**Documentation**

- `README.md` — command reference, technical spec, quick start
- `FORMAL_FOUNDATION.md` — mathematical and cryptographic specification
- `COMPARISON.md` — feature matrix vs. traditional audit logs, checksums, LangChain guardrails
- `docs/CASE_STUDY.md` — the $400K hallucination scenario (without vs. with Coh)
- `docs/00-purpose-and-scope.md` — validator purpose and V1 scope
- `docs/01-canonical-data-model.md` — 4-layer data model and field reference
- `docs/02-verifier-ordering.md` — 6-step verification order and rationale
- `docs/03-chain-and-slab-laws.md` — chain digest rule, slab laws, reject code taxonomy
- `docs/04-merkle-challenge-flow.md` — Merkle tree construction and audit challenge flow
- `docs/05-cli-usage.md` — full CLI reference with examples
- `docs/06-test-vectors.md` — fixture descriptions and oracle guidance
- `docs/07-lean-to-rust-traceability.md` — Lean 4 theorem to Rust enforcement mapping
- `WEDGE_CHECKLIST.md` — V1 stabilization checklist (all 10 steps complete)
- `ROADMAP.md` — V2–V4 roadmap and research directions

**Integration Templates**

- `examples/integrations/generic_agent_loop.rs` — generic LLM agent integration
- `examples/integrations/openai_function_calling.rs` — OpenAI function-call wrapper

**Showcase Demo**

- `examples/showcase.rs` — 60-second cinematic demo: hallucination breach + circuit breaker
  (`cargo run --example showcase -p coh-core --release`)

**Formal Verification**

- Accounting law (`IsLawful`) proved in Lean 4: [github.com/noeticanlabs/coh-lean](https://github.com/noeticanlabs/coh-lean)
- `lawful_composition` theorem: aggregate slab law follows from per-step law

---

[0.1.0]: https://github.com/noeticanlabs/Coh-wedge/releases/tag/v0.1.0
