# Documentation Gap Audit

> Phase A: Truth reconciliation before documentation writing

---

## 1. CLI Commands (coh-cli)

### 1.1 verify-micro
- **Current Behavior**: Loads JSON file, verifies single MicroReceiptWire
- **Current Docs**: `coh-node/docs/05-cli-usage.md` - exists
- **Status**: ✅ Matches implementation

### 1.2 verify-chain
- **Current Behavior**: Loads JSONL file, verifies list of receipts, returns chain validation result  
- **Current Docs**: `coh-node/docs/05-cli-usage.md` - exists
- **Status**: ✅ Matches implementation

### 1.3 build-slab
- **Current Behavior**: Takes JSONL input + --out flag, outputs SlabReceipt to file
- **Current Docs**: `coh-node/docs/05-cli-usage.md` - exists
- **Status**: ✅ Matches implementation

### 1.4 verify-slab
- **Current Behavior**: Loads JSON file, verifies standalone slab
- **Current Docs**: `coh-node/docs/05-cli-usage.md` - exists
- **Status**: ✅ Matches implementation

---

## 2. Exit Code Contract

### CLI Exit Codes
| Code | Meaning | Implementation |
|------|---------|----------------|
| 0 | ACCEPT | `if res.is_accept() { 0 } else { 1 }` (line 169) |
| 1 | REJECT | All non-accept decisions |
| 2 | MALFORMED | File load errors (line 51, 63, 75) |
| 3 | ERROR | Catch-all errors (line 85, 113) |

**Current Docs**: `coh-node/docs/05-cli-usage.md` - exit codes 0-4 documented

**Gap**: ⚠️ **MISMATCH** - Docs list 0-4 but implementation uses 0-3 only

**Resolution needed**: Either:
1. Update docs to match implementation (0=accept, 1=reject, 2=malformed, 3=error)
2. Add exit codes 4+ for schema/version to match docs

---

## 3. Wire Schemas

### 3.1 MicroReceiptWire
- **Fields**: schema_id, version, object_id, canon_profile_hash, policy_hash, step_index, state_hash_prev, state_hash_next, chain_digest_prev, chain_digest_next, metrics
- **Current Docs**: Partially in `coh-node/docs/01-canonical-data-model.md`
- **Status**: ⚠️ Partial - field definitions exist but no formal JSON schema

### 3.2 SlabReceiptWire  
- **Fields**: schema_id, version, object_id, canon_profile_hash, policy_hash, range_start, range_end, micro_count, chain_digest_prev, chain_digest_next, state_hash_first, state_hash_last, merkle_root, summary
- **Current Docs**: Partially in `coh-node/docs/01-canonical-data-model.md`
- **Status**: ⚠️ Partial - field definitions exist but no formal JSON schema

---

## 4. Result Structs

### 4.1 VerifyMicroResult / VerifyChainResult / VerifySlabResult
- **Current implementation**: decision (Accept/Reject), code (RejectCode), message, step_index, object_id, chain_digest_next
- **Current Docs**: Not formally documented
- **Status**: ⚠️ Missing - needs JSON output schema

### 4.2 Decision enum
- **Values**: Accept, Reject
- **Current implementation**: From types.rs
- **Status**: ✅ Implemented but not in user docs

---

## 5. Sidecar API Routes

### 5.1 /v1/verify-micro
- **Current behavior**: POST accepts MicroReceiptWire, returns ApiResponse<VerifyMicroResult>
- **Implementation**: routes.rs line ~32
- **Current Docs**: ❌ None
- **Status**: ❌ Missing

### 5.2 /v1/verify-chain  
- **Current behavior**: POST accepts array, returns ApiResponse<VerifyChainResult>
- **Implementation**: routes.rs line ~54
- **Current Docs**: ❌ None
- **Status**: ❌ Missing

### 5.3 /v1/execute-verified
- **Current behavior**: POST accepts action, returns result
- **Implementation**: routes.rs line ~75
- **Current Docs**: ❌ None  
- **Status**: ❌ Missing

### Error Codes (sidecar)
- CohErrorCode: E001 (verify-micro), E003 (chain), E004 (slab), E005 (execute)
- **Current Docs**: ❌ None
- **Status**: ❌ Missing

---

## 6. Python Bindings (coh-python)

### Exposed functions (from lib.rs):
- normalize, verify, verify_chain, build_slab, verify_slab, hash, compare, assert_equivalent
- **Current Docs**: ❌ None  
- **Status**: ❌ Missing user guide

### Input forms accepted:
- Need to verify actual API surface

---

## 7. Dashboard (coh-dashboard)

### 7.1 App layout
- **Current implementation**: React App.jsx with scenario-driven fixture loading
- **Current Docs**: Minimal README only
- **Status**: ⚠️ Needs architecture doc

### 7.2 Data flow
- Fixture source → parseJsonLines → normalizeStep → deriveChainBreak/deriveSlabCheck
- **Current Docs**: ❌ None
- **Status**: ❌ Missing

### 7.3 Scenarios  
- valid, invalid_state_link, reject_chain_digest, reject_policy_violation, etc.
- **Current Docs**: Not documented
- **Status**: ❌ Missing

---

## 8. Lean Correspondence

### Invariants in coh-lean:
- Location: `coh-lean/Coh/` 
- **Current Docs**: Individual theorem files exist
- **Status**: ⚠️ No mapping table to implementation

---

## 9. Project Metadata

### CHANGELOG.md
- **Current**: Exists, covers V1 release
- **Gap**: Missing recent changes (demo_test.sh, UI tests, CI fixes)

### ROADMAP.md
- **Current**: V1 shipped, V2-V4 planned
- **Status**: ✅ Fine, refresh after priorities

---

## Summary: Gap Severity

| Area | Severity | Action |
|------|-----------|--------|
| Exit codes | HIGH | Reconcile implementation vs docs |
| Sidecar API | HIGH | Document routes and error codes |
| Python API | HIGH | Create usage guide |
| CLI exit codes | MEDIUM | Update docs to match 0-3 |
| Receipt schemas | MEDIUM | Add formal JSON schema |
| Dashboard | MEDIUM | Document architecture |
| Lean mapping | LOW | Create correspondence table |
| CHANGELOG | LOW | Refresh after fix |

---

## Recommended Execution Order

1. **Fix exit code mismatch** (HIGH priority) - decide whether to update code or docs
2. **Document sidecar API** (HIGH) - routes and error contracts  
3. **Document Python API** (HIGH) - usage guide
4. Add JSON schemas for wire types
5. Document dashboard architecture  
6. Create Lean correspondence table
7. Refresh CHANGELOG/ROADMAP