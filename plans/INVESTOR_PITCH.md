# Coh - 2-Minute Investor Pitch

## The Problem

AI systems fail silently. They:
- Report state transitions that never happened
- Skip required intermediate steps  
- Produce impossible accounting updates
- Continue execution after integrity breaks

**Result**: You can't trust AI outputs in production.

## The Solution

**Coh** is a deterministic execution boundary that ensures only valid state transitions can occur.

Instead of accepting an LLM's narration of what happened, Coh requires machine-checkable receipts. Invalid workflows get explicit `REJECT` decisions with stable, auditable surfaces.

## The Wedge

```
Untrusted AI output → Coh Verifier → ACCEPT / REJECT
                                        ↓
                              Only valid transitions
                              reach production state
```

## Key Differentiators

| Capability | What It Means |
|------------|----------------|
| **Deterministic** | Same input → same decision, every time |
| **Fast** | 70k+ ops/sec, p99 < 130µs |
| **Measurable** | 0% false accepts on adversarial test corpus |
| **Composable** | Works with chains, workflows, slabs, sidecar |

## Evidence

- **Throughput**: 70k-90k ops/sec (single-threaded)
- **Concurrency**: 300k+ ops/sec (500 threads)  
- **False Accept Rate**: 0% (observed - 0 invalid accepted)
- **False Reject Rate**: 0% (observed - 0 valid rejected)
- **Latency p99**: < 130µs under load

**Reproducibility**: All metrics captured with full hardware spec, build flags, and test dataset for audit.

## Target Markets

1. **Enterprise AI ops** - Any system where AI actions must be verifiable before commit
2. **Financial services** - Require deterministic audit trails
3. **Industrial maintenance** - Ensure work orders follow correct procedures
4. **Agent frameworks** - Need trusted execution boundaries

## Why Now

- LLMs are entering production workflows
- No existing solution for deterministic AI verification
- Regulatory pressure for AI auditability
- Enterprise demand for "trust but verify"

---

**Status**: Investor-ready for technical pre-seed conversation.

Run the benchmark:
```bash
cd coh-node && cargo run --release --example enterprise_benchmark
```

Full metrics and reproducibility block included in output.