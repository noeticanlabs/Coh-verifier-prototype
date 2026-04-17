# Documentation & Specification Plan

## Phase 1: Core Documentation Review & Updates

### 1.1 Verify Core API Documentation 
**Status**: Partial - existing but needs expansion
- `verify_micro.rs` - single receipt verification
- `verify_chain.rs` - contiguous chain verification 
- `verify_slab.rs` - standalone slab validation
- `build_slab.rs` - chain to slab aggregation
- `canon.rs` - canonicalization engine

### 1.2 Receipt Schema Documentation
**Status**: Missing formal documentation
- `MicroReceiptWire` (JSON input)
- `SlabReceiptWire` (JSON input)  
- `MetricsWire` (v_pre, v_post, spend, defect)
- `SlabSummaryWire` (totals)

### 1.3 CLI Command Documentation
**Status**: Partial - code has doc comments but needs user guide
- `verify-micro` - single step verification
- `verify-chain` - JSONL chain verification
- `build-slab` - aggregation with Merkle root
- `verify-slab` - standalone slab check
- `--format json` flag

### 1.4 Exit Code Contract
**Status**: Missing formal spec
- Code 0: ACCEPT
- Code 1: Chain break detected
- Code 2: Invalid input/malformed
- Code 3: Policy violation
- Code 4: Schema/version mismatch

---

## Phase 2: Integration & Usage Guides

### 2.1 Python Bindings
**Status**: Missing user guide
- `coh-python` crate usage
- PyO3 integration patterns

### 2.2 Sidecar API
**Status**: Missing documentation
- REST routes in `coh-sidecar/src/routes.rs`
- HTTP API contract

### 2.3 Dashboard UI
**Status**: Needs documentation
- React component architecture
- Data flow: fixtures → JSON → verification
- Demo scenarios

### 2.4 Integration Templates
**Status**: Existing examples, needs guide
- OpenAI function calling
- Generic agent loop
- LangChain integration points

---

## Phase 3: Formal Specification

### 3.1 Unified Requirements Spec
Create single source of truth:
- Protocol version and backward compatibility
- Security assumptions
- Performance guarantees (77K steps/sec)
- Integrity guarantees

### 3.2 Formal Verification Mapping
**Status**: coh-lean exists, needs mapping doc
- Invariants in Lean 4
- Code correspondence table
- Theorem statements

---

## Phase 4: Project Metadata

### 4.1 CHANGELOG.md
Needs update with:
- Demo test additions
- UI test infrastructure
- CI improvements

### 4.2 ROADMAP.md  
Review V2-V4 priorities based on production feedback

---

## Deliverables

1. **API Reference** - Auto-generated from source docs
2. **Receipt Schema Spec** - JSON schema + examples  
3. **Integration Guide** - Python, OpenAI, LangChain
4. **Requirements Document** - Single specification source
5. **Updated CHANGELOG** - Recent changes
6. **Updated ROADMAP** - Prioritized features