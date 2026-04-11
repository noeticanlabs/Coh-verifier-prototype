import coh
import json

# Sample receipt as a native Python DICTIONARY (Polymorphic support)
receipt = {
    "schema_id": "coh.receipt.micro.v1",
    "version": "1.0.0",
    "object_id": "agent.workflow.demo",
    "canon_profile_hash": "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
    "policy_hash": "0" * 64,
    "step_index": 0,
    "state_hash_prev": "1" * 64,
    "state_hash_next": "2" * 64,
    "chain_digest_prev": "0" * 64,
    "chain_digest_next": "76114b520738a80e18048c2a37734c97b17d6f7e06f94393bbce7949bb87381c",
    "metrics": {
        "v_pre": "100",
        "v_post": "88",
        "spend": "12",
        "defect": "0"
    }
}

chain_jsonl = """{"schema_id":"coh.receipt.micro.v1","version":"1.0.0","object_id":"obj_123","canon_profile_hash":"4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09","policy_hash":"0000000000000000000000000000000000000000000000000000000000000000","step_index":0,"state_hash_prev":"0000000000000000000000000000000000000000000000000000000000000000","state_hash_next":"0000000000000000000000000000000000000000000000000000000000000001","chain_digest_prev":"0000000000000000000000000000000000000000000000000000000000000000","chain_digest_next":"d6f3b24b580b5d4b3f3ee683ecf02ef47e42837cc0d7c13daab4e082923a5149","metrics":{"v_pre":"100","v_post":"80","spend":"15","defect":"0"}}
{"schema_id":"coh.receipt.micro.v1","version":"1.0.0","object_id":"obj_123","canon_profile_hash":"4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09","policy_hash":"0000000000000000000000000000000000000000000000000000000000000000","step_index":1,"state_hash_prev":"0000000000000000000000000000000000000000000000000000000000000001","state_hash_next":"0000000000000000000000000000000000000000000000000000000000000002","chain_digest_prev":"d6f3b24b580b5d4b3f3ee683ecf02ef47e42837cc0d7c13daab4e082923a5149","chain_digest_next":"1fa90ecefbd25df4c47848c66e919ca5676b21255173c850cc3110df6ee51114","metrics":{"v_pre":"80","v_post":"60","spend":"20","defect":"0"}}"""

slab_data = {
  "schema_id": "coh.receipt.slab.v1",
  "version": "1.0.0",
  "object_id": "obj_123",
  "canon_profile_hash": "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
  "policy_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "range_start": 0,
  "range_end": 1,
  "micro_count": 2,
  "chain_digest_prev": "0000000000000000000000000000000000000000000000000000000000000000",
  "chain_digest_next": "1fa90ecefbd25df4c47848c66e919ca5676b21255173c850cc3110df6ee51114",
  "state_hash_first": "0000000000000000000000000000000000000000000000000000000000000000",
  "state_hash_last": "0000000000000000000000000000000000000000000000000000000000000002",
  "merkle_root": "5c6e9d8f5f8f2d29fdc4871d96b3018f7cae37729eaf76f3f944897b104ce650",
  "summary": {
    "total_spend": "35",
    "total_defect": "0",
    "v_pre_first": "100",
    "v_post_last": "60"
  }
}

print("--- Testing Coh V1 Unified Python API ---")

try:
    # 1. Normalize
    result = coh.normalize(receipt)
    print(f"[1] Normalized Hash: {result.hash}")

    # 2. Verify Micro
    coh.verify(receipt)
    print("[2] Verify Micro: Success")

    # 3. Verify Chain (JSONL)
    print("[3] Verifying Chain (JSONL)...")
    c_res = coh.verify_chain(chain_jsonl)
    print(f"    Decision: {c_res['decision']}, Steps: {c_res['steps_verified']}")

    # 4. Build Slab from List
    print("[4] Building Slab from list of dicts...")
    chain_list = [json.loads(l) for l in chain_jsonl.splitlines()]
    s_res = coh.build_slab(chain_list)
    print(f"    Decision: {s_res['decision']}, Merkle Root: {s_res.get('merkle_root')}")

    # 5. Verify Slab
    print("[5] Verifying Slab...")
    vs_res = coh.verify_slab(slab_data)
    print(f"    Decision: {vs_res['decision']}, Range: {vs_res['range_start']}-{vs_res['range_end']}")

    # 6. Test Exception Handling
    print("[6] Testing CohVerificationError...")
    tampered = receipt.copy()
    tampered["metrics"]["spend"] = "999"
    try:
        coh.verify(tampered)
    except coh.CohVerificationError as e:
        print(f"    Caught expected error: {e}")

    print("\n--- V1 API Upgraded Demo Complete ---")

except Exception as e:
    print(f"Error during execution: {e}")
    import traceback
    traceback.print_exc()
