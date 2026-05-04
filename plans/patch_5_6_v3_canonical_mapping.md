# Patch 5: V3 Canonical Mapping
# Patch 6: Computed Sequence Accumulator

## Context

The accounting law is now sealed (Patches 1-3 integrated through shared kernel). Next attack surface is cryptographic semantic binding in V3.

---

## Patch 5: V3 Canonical Mapping

### Attack Question

> Can objective_result, sequence_valid, override_applied, profile, or governance metadata be changed without changing the digest?

If YES: V3 is still projected (attacker-writable).  
If NO: V3 is canonical (digest-bound).

---

### Field Binding Classification

| Field | Current State | Binding Target | Accept/Reject Impact |
|-------|--------------|----------------|---------------------|
| `objective_result` | bool checkable | MUST DIGEST-BIND | RejectPolicyViolation if violated |
| `sequence_valid` | bool self-attested | REPLACE with computed | RejectPolicyViolation if false |
| `override_applied` | bool writable | MUST DIGEST-BIND | Bypasses crypto check if true |
| `canon_profile_hash` | in digest | Already bound | Selects law set |
| `policy_hash` | in digest | Already bound | Policy constraints |
| `chain_digest_prev/next` | in digest | Already bound | Chain continuity |

### V3 Canonical Prehash Structure

```rust
/// V3 canonical prehash - all fields that affect accept/reject
#[derive(Clone, Debug)]
pub struct MicroReceiptV3Prehash {
    // Base fields (already in digest)
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub step_index: u64,
    pub state_hash_prev: Hash32,
    pub state_hash_next: Hash32,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    
    // Accounting (already in digest)
    pub v_pre: u128,
    pub v_post: u128,
    pub spend: u128,
    pub defect: u128,
    pub authority: u128,
    pub delta_hat: u128,
    
    // V3 NEW - must be digest-bound
    pub objective_result: Option<ObjectiveResult>,
    pub sequence_accumulator: Option<Hash32>,  // NEW: replaces sequence_valid bool
    pub override_applied: bool,                  // NEW: bound in digest
}
```

### Implementation Steps

1. **Add MicroReceiptV3Prehash to types_v3.rs**
   - Include all V3 Transition Contract fields
   - Derive canonical prehash method

2. **Implement V3 canonical digest**
   ```rust
   impl MicroReceiptV3Prehash {
       pub fn canonical_digest(&self) -> Hash32 {
           let mut hasher = Sha256::new();
           hasher.update(b"COH_V3_CANONICAL");
           
           // Base fields
           hasher.update(self.schema_id.as_bytes());
           hasher.update(self.version.as_bytes());
           hasher.update(self.object_id.as_bytes());
           hasher.update(self.canon_profile_hash.0);
           hasher.update(self.policy_hash.0);
           hasher.update(self.step_index.to_be_bytes());
           
           // Accounting
           hasher.update(self.v_pre.to_be_bytes());
           hasher.update(self.v_post.to_be_bytes());
           hasher.update(self.spend.to_be_bytes());
           hasher.update(self.defect.to_be_bytes());
           hasher.update(self.authority.to_be_bytes());
           hasher.update(self.delta_hat.to_be_bytes());
           
           // State binding
           hasher.update(self.state_hash_prev.0);
           hasher.update(self.state_hash_next.0);
           hasher.update(self.chain_digest_prev.0);
           hasher.update(self.chain_digest_next.0);
           
           // V3 Transition Contract fields (NEW)
           if let Some(ref obj) = self.objective_result {
               hasher.update(serde_json::to_vec(obj).unwrap());
           }
           if let Some(seq) = self.sequence_accumulator {
               hasher.update(seq.0);
           }
           hasher.update([self.override_applied as u8]);
           
           Hash32(hasher.finalize().into())
       }
   }
   ```

3. **Update verify_micro_v3.rs**
   - Compute V3 canonical digest from prehash
   - Verify V3 metadata mutation changes digest
   - Reject if any V3 field mutates without digest change

---

## Patch 6: Computed Sequence Accumulator

### Replace Boolean sequence_valid

Current (vulnerable):
```rust
pub sequence_valid: bool  // Self-attested!
```

Target (digest-bound):
```rust
pub sequence_accumulator: Option<Hash32>  // Computed
```

### Sequence Accumulator Algorithm

Already implemented in sequence_accumulator.rs:

```rust
pub fn compute_sequence_accumulator(
    prev_guard: Hash32,
    receipt_digest: Hash32,
    step_index: u64,
    state_pre: Hash32,
    state_post: Hash32,
) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(b"COH_SEQUENCE_V1");
    hasher.update(prev_guard.0);
    hasher.update(receipt_digest.0);
    hasher.update(step_index.to_be_bytes());
    hasher.update(state_pre.0);
    hasher.update(state_post.0);
    Hash32(hasher.finalize().into())
}
```

### Integration Steps

1. **Add sequence_accumulator field to MicroReceiptV3Wire**
   - Make sequence_valid optional (for backward compat)
   - Add sequence_accumulator: Option<String>

2. **Update verify_micro_v3.rs verification**
   - Accept optional boolean for legacy
   - Require computed accumulator for V3 canonical
   - Verify: compute_sequence_accumulator matches claimed

3. **Tamper Test**
   ```rust
   #[test]
   fn test_v3_tamper_sequence_accumulator() {
       // Given: valid receipt with sequence_accumulator
       let mut wire = valid_v3_wire();
       wire.sequence_accumulator = Some(valid_accumulator);
       
       // When: attacker flips sequence_valid
       wire.sequence_valid = false;
       
       // Then: digest MUST change
       let digest1 = compute_v3_digest(&wire);
       let digest2 = compute_v3_digest(&modified);
       assert_ne!(digest1, digest2);
   }
   ```

---

## Tamper Tests Required

### Test 1: objective_result Digest Binding
```rust
#[test]
fn test_objective_result_digest_binding() {
    let mut wire = valid_v3_wire();
    let digest1 = compute_v3_digest(&wire);
    
    // Attacker changes objective_result without changing digest
    wire.objective_result = Some(ObjectiveResult::Violated(
        ObjectiveTarget::MinimizeSpend
    ));
    let digest2 = compute_v3_digest(&wire);
    
    // MUST be rejected - digest must change
    assert_ne!(digest1, digest2, 
        "objective_result MUST affect digest");
}
```

### Test 2: override_applied Digest Binding
```rust
#[test]
fn test_override_applied_digest_binding() {
    let mut wire = valid_v3_wire();
    let digest1 = compute_v3_digest(&wire);
    
    // Attacker sets override to bypass
    wire.override_applied = true;
    let digest2 = compute_v3_digest(&wire);
    
    // MUST be rejected - digest must change
    assert_ne!(digest1, digest2,
        "override_applied MUST affect digest");
}
```

### Test 3: sequence_accumulator Verification
```rust
#[test]
fn test_sequence_accumulator_verification() {
    // Given: receipt with valid sequence
    let prev_guard = Hash32([0; 32]);
    let receipt_digest = compute_v3_digest(&valid_wire);
    let claimed_seq = compute_sequence_accumulator(
        prev_guard,
        receipt_digest,
        step_index,
        state_pre,
        state_post,
    );
    
    // When: verify
    let valid = verify_sequence_accumulator(
        claimed_seq,
        prev_guard,
        receipt_digest,
        step_index,
        state_pre,
        state_post,
    );
    
    // Then: passes
    assert!(valid);
}
```

---

## Files to Modify

| File | Change |
|------|--------|
| `types_v3.rs` | Add MicroReceiptV3Prehash, sequence_accumulator field |
| `verify_micro_v3.rs` | Use prehash, bind V3 fields, verify accumulator |
| `v3_canonical.rs` | May need alignment with new prehash |

---

## Verification Flow (Post-Patch)

```
verify_micro_v3(wire)
    ├── Parse to MicroReceiptV3
    ├── Compute MicroReceiptV3Prehash
    │   └── Include objective_result, override_applied
    ├── Compute V3 canonical digest from prehash
    │   └── All V3 Transition Contract fields bound
    ├── Verify sequence_accumulator (if provided)
    │   └── compute_sequence_accumulator(prev, digest, step, state_pre, state_post)
    └── Decision: Accept/Reject
```

---

## Risk Closure

| Risk | Before | After |
|------|--------|-------|
| objective_result mutable | YES (no digest) | NO (in digest) |
| sequence_valid self-attested | YES (bool) | NO (computed) |
| override_applied mutable | YES (no digest) | NO (in digest) |
| V3 fields affect digest | Unknown | Verified by tamper test |

This closes the cryptographic semantic binding gap.