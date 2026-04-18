# Option C: Live Workflow Integration — Sidecar LLM Proxy

## Overview

Build an HTTP sidecar that acts as an OpenAI-compatible proxy between LLM clients (apps, agents, SDKs) and upstream LLM APIs. For each LLM interaction, the sidecar:

1. Forwards the request to the upstream LLM
2. Captures the response (including token usage, finish_reason)
3. Generates a micro-receipt encoding the response metadata
4. Verifies the receipt via Coh's `verify_micro()`
5. On ACCEPT: returns the original response to the caller
6. On REJECT: returns an error, blocks downstream action

## System Architecture

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                              LLM Client / Agent                                 │
│                         (OpenAI SDK, LangChain, etc.)                          │
└─────────────────────────────────┬──────────────────────────────────────────────┘
                                  │ HTTP/JSON
                                  │ POST /v1/chat/completions
                                  ▼
┌────────────────────────────────────────────────────────────────────────────────┐
│                            Coh Sidecar (LLM Proxy)                             │
│  ┌─────────────────┐   ┌──────────────────┐   ┌─────────────────────────────┐  │
│  │  Request        │   │  Receipt         │   │  Verifier                  │  │
│  │  Forwarder      │──▶│  Generator       │──▶│  (verify_micro)            │  │
│  │                 │   │                  │   │                             │  │
│  └─────────────────┘   └──────────────────┘   └─────────────────────────────┘  │
│                               │                        │                        │
│                               │ ACCEPT                 │ REJECT                 │
│                               ▼                        ▼                        │
│  ┌─────────────────┐   ┌──────────────────┐   ┌─────────────────────────────┐  │
│  │  Response       │   │  Receipt         │   │  Error Response             │  │
│  │  Passthrough    │◀──│  Appended        │◀──│  (blocked)                  │  │
│  │                 │   │  (metadata)      │   │                             │  │
│  └─────────────────┘   └──────────────────┘   └─────────────────────────────┘  │
│                                                                                 │
│  ┌──────────────────────────┐  ┌────────────────────────────────────────────┐  │
│  │  State Tracker           │  │  Metrics / Dashboard                      │  │
│  │  (chain_digest, v_pre)   │  │  (accepts, rejects, latency p95)          │  │
│  └──────────────────────────┘  └────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  │ Upstream LLM
                                  ▼ (optional — can be simulated)
                    ┌─────────────────────────────┐
                    │  OpenAI / Anthropic / etc.  │
                    └─────────────────────────────┘
```

## API Surface

### 1. OpenAI-Compatible Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | /v1/chat/completions | Chat completions (main entry) |
| POST | /v1/completions | Legacy completions |
| GET | /v1/models | List available models |
| GET | /health | Liveness probe |

### 2. Coh-Specific Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /coh/verify/{step_index} | Verify a specific receipt |
| GET | /coh/chain | Current chain digest + length |
| GET | /coh/stats | Accept/reject counts, latency percentiles |
| POST | /coh/reset | Reset chain state (new workflow) |

## Request/Response Schemas

### Chat Completions Request (passthrough)

```json
{
  "model": "gpt-4",
  "messages": [{"role": "user", "content": "Summarize the Q3 financial report"}],
  "max_tokens": 500,
  "temperature": 0.7
}
```

### Chat Completions Response (original + receipt metadata)

On ACCEPT (200 OK):

```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1700000000,
  "model": "gpt-4",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "The Q3 financial report shows..."
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 120,
    "completion_tokens": 85,
    "total_tokens": 205
  },
  "coh_receipt": {
    "schema_id": "coh.receipt.micro.v1",
    "version": "1.0.0",
    "object_id": "llm.chat.0",
    "canon_profile_hash": "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
    "policy_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "step_index": 0,
    "step_type": "llm_chat",
    "signatures": [{"signature": "sig-...", "signer": "coh-sidecar", "timestamp": 1700000000}],
    "state_hash_prev": "0000000000000000000000000000000000000000000000000000000000000000",
    "state_hash_next": "a1b2c3d4e5f6...",
    "chain_digest_prev": "0000000000000000000000000000000000000000000000000000000000000000",
    "chain_digest_next": "deadbeef...",
    "metrics": {
      "v_pre": "1000",
      "v_post": "795",
      "spend": "205",
      "defect": "0",
      "authority": "1"
    }
  },
  "coh_decision": "ACCEPT"
}
```

On REJECT (403 Forbidden):

```json
{
  "error": {
    "message": "Coh verification failed: policy violation at step 0. Token spend (205) exceeds v_pre (100) + defect (0) + authority (0). Receipt rejected.",
    "type": "coh_verification_error",
    "code": "RejectPolicyViolation",
    "coh_decision": "REJECT",
    "coh_receipt": { /* the failing receipt */ }
  }
}
```

## Mapping: LLM Response → Receipt Fields

| LLM Field | Receipt Field | Notes |
|-----------|---------------|-------|
| `usage.prompt_tokens` | `metrics.spend` | Prompt tokens = compute cost |
| `usage.completion_tokens` | (implicit) | Added to total spend |
| `usage.total_tokens` | `metrics.spend` | Total tokens consumed |
| `finish_reason == "stop"` | `metrics.authority = 1` | Success |
| `finish_reason != "stop"` | `metrics.defect = 1` | Non-terminal (length, etc) |
| `model` | `object_id` | e.g., "llm.gpt-4.0" |
| `created` (timestamp) | `signatures[0].timestamp` | Unix epoch |
| (counter) | `step_index` | Monotonically increasing |
| (session state) | `state_hash_prev/next` | Tracks conversation state |
| (session state) | `chain_digest_prev/next` | Chain of LLM calls |

## Reject Codes and Client Actions

| RejectCode | Meaning | Client Action |
|------------|---------|---------------|
| RejectMissingSignature | Sidecar misconfigured | Retry after config fix |
| RejectChainDigest | State corruption | Reset session, restart |
| RejectPolicyViolation | Spend exceeds budget | Reduce max_tokens, retry |
| RejectStateHashLink | Client spoofed history | Reset session |
| RejectSchema | Invalid receipt format | Bug in sidecar — alert |

## State Management

- **Per-session state**: Chain digest + v_pre (token budget) stored in-memory per session ID
- **Session ID**: Derived from `X-Session-Token` header or cookie; defaults to "default"
- **Budget initialization**: v_pre defaults to 1000 tokens; configurable via `X-Coh-Budget` header
- **Reset**: POST /coh/reset clears state for a session

## Configuration (Environment Variables)

| Variable | Default | Description |
|----------|---------|-------------|
| COH_SIDECAR_UPSTREAM_URL | http://localhost:11434 | Upstream LLM (Ollama, OpenAI, etc.) |
| COH_SIDECAR_INITIAL_BUDGET | 1000 | Starting token budget per session |
| COH_SIDECAR_PORT | 3000 | HTTP listen port |
| COH_SIDECAR_LOG_LEVEL | info | Tracing level |

## Metrics exposed at GET /coh/stats

```json
{
  "sessions_active": 3,
  "total_requests": 1250,
  "accepts": 1198,
  "rejects": 52,
  "false_accept_rate": 0.0,
  "false_reject_rate": 0.0,
  "latency_p50_ms": 12.3,
  "latency_p95_ms": 45.7,
  "latency_p99_ms": 89.2,
  "upstream_latency_p95_ms": 2100.0
}
```

## Implementation Steps

1. **Extend existing sidecar**: Add request forwarder + receipt generator + verifier
2. **New module**: `coh-sidecar/src/llm_proxy.rs` — handles OpenAI compatibility
3. **Session state**: `coh-sidecar/src/session.rs` — HashMap<session_id, SessionState>
4. **Receipt generator**: `coh-sidecar/src/receipt.rs` — maps LLM response → MicroReceiptWire
5. **Integration test**: Run benchmark against live Ollama instance (or mock)

## Comparison with Options A & B

| Criterion | Option A (Sim) | Option B (Logs) | Option C (Live) |
|-----------|----------------|-----------------|-----------------|
| Validation Type | Synthetic | Retrospective | Real-time |
| Latency Impact | None | None | ~1-5ms overhead |
| False Accept/Reject | 0% | (depends on logs) | 0% expected |
| Production Ready | No | Maybe | **Yes** |
| Investor Story | "Works in test" | "Works on logs" | "Works in prod" |

## Deliverables

- [ ] Updated sidecar with OpenAI-compatible endpoints
- [ ] Receipt generator mapping LLM responses to MicroReceiptWire
- [ ] Session state tracking (chain + budget)
- [ ] Inline verifier — blocks on REJECT
- [ ] /coh/stats endpoint for monitoring
- [ ] Integration test (Ollama or mock)