# NPE Library Stack Integration Plan

## 1. Architectural Strategy: Split Across Crates

As discussed, we will adopt the "Split across crates" architecture. This maintains the clean separation between proposal generation (exploration) and verifier authority (admissibility).

*   **`coh-core`**: Will remain the trusted minimal verifier kernel. It will not receive heavy NPE search dependencies. It maintains deterministic primitives, verifier boundaries, reject codes, and canonical serialization using exact arithmetic (`num-rational`, `num-bigint`).
*   **`coh-genesis`**: Will act as the primary target for the NPE loop expansion. It currently houses the Genesis / proposal-side logic and closure examples. The NPE loop encompasses `generate -> mutate -> score -> attempt proof -> ingest receipt -> refine`.

A new crate, `coh-npe`, will only be extracted later once the API within `coh-genesis` stabilizes.

## 2. Dependency & Feature Flag Layout (`coh-genesis/Cargo.toml`)

We will introduce the new libraries into `coh-genesis` guarded by feature flags. This allows consumers to opt-in to advanced search or storage capabilities without bloat.

### Recommended Additions:
```toml
[dependencies]
# Existing (or needed for deterministic generation/serialization)
serde = { workspace = true }
serde_json = { workspace = true }
schemars = "0.8"
sha2 = { workspace = true }
hex = { workspace = true }
rand = "0.8"
rand_chacha = "0.3"

# New NPE Extensions (Optional via features)
petgraph = { version = "0.6", optional = true }
egg = { version = "0.9", optional = true }
rusqlite = { version = "0.31", features = ["bundled"], optional = true }
rayon = { version = "1", optional = true }
nalgebra = { version = "0.33", optional = true }
ndarray = { version = "0.16", optional = true }
z3 = { version = "0.12", optional = true }

[features]
default = []
npe-graph = ["petgraph"]
npe-rewrite = ["egg"]
npe-store = ["rusqlite"]
npe-parallel = ["rayon"]
npe-smt = ["z3"]
npe-numeric = ["nalgebra", "ndarray"]
```

## 3. Proposal Lineage Strategy (`petgraph`)

The NPE will construct a directed proposal graph representing the exploration space.
*   **Nodes ($G_{proposal}$)**: Represent proposal states (e.g., semantic envelopes, math forms, lean code blocks).
*   **Edges ($E$)**: Represent mutation or refinement transitions (e.g., `mutate`, `simplify`, `rewrite`).
*   **Edge Labels**: Contain the mutation type, advisory score, verifier verdict, and the deterministic receipt.

This graph structure ensures closure visibility (which paths lead to ACCEPT vs REJECT) and allows the engine to backtrack or branch optimally.

## 4. Equivalence and Rewrite Strategy (`egg`)

We will use equality saturation (`egg`) to compress the proposal space.
*   Equivalent mathematical or structural forms ($p_1 \sim p_2$) are grouped into an e-graph.
*   The NPE will extract the lowest-cost representative candidate before submitting it to the Coherence verifier.
*   This prevents the NPE from exploring redundant, isomorphic paths, significantly speeding up loop convergence.

## 5. Deterministic Receipt and Storage Strategy (`rusqlite`)

To satisfy Lemma 1 (Deterministic replay), all proposals and receipts must be stored predictably.
*   **Database**: `rusqlite` will store the memory manifold. Tables will include `proposals`, `receipts`, `edges`, `verdicts`, `scores`, `proof_attempts`, and `semantic_envelopes`.
*   **Canonical Ordering**: `BTreeMap` will be preferred over `HashMap` for any memory structures serialized to JSON.
*   **Hashing ($H_{n+1}$)**: Will use SHA256 over `tag || H_n || JCS(r_n)`.

## 6. Execution and Parallelism (`rayon`)

*   **Rule**: Parallel execution must not break reproducibility.
*   **Workflow**:
    1. Generate a deterministic batch of candidates (using `rand_chacha`).
    2. Score candidates in parallel (`rayon`).
    3. Sort candidates canonically based on score/hash.
    4. Verify proposals sequentially via the Coh verifier.
    5. Commit the receipt and update the memory manifold.

## 7. Failure Mode Mitigations

1.  **Floating Score becomes Verifier Truth**: Ensure floating-point scores (`f64`, embeddings) are *strictly* advisory for ranking. Final acceptance relies entirely on integer/rational math via the Coh Verifier (`RV(p) = ACCEPT`).
2.  **Nondeterministic Proposal Order**: Prevent race conditions during parallel scoring by forcing a canonical sort before sequential verification.
3.  **Solver Trust Leak**: If Z3/SMT is used, it only *suggests* candidates. It does not dictate correctness.
4.  **Memory Manifold Poisoning**: Track explicit rejection reasons (e.g., `REJECTED_POLICY`, `REJECTED_OVERFLOW`) in the SQLite database so the NPE learns *why* paths failed, rather than just that they failed.

## 8. Phased Implementation Roadmap

*   **Phase 1**: Add dependencies and feature flags to `coh-node/crates/coh-genesis/Cargo.toml`.
*   **Phase 2**: Implement the deterministic Random generation (`rand_chacha`) and parallel scoring/sorting loop (`rayon`).
*   **Phase 3**: Integrate `petgraph` to construct the in-memory proposal lineage graph.
*   **Phase 4**: Implement `rusqlite` storage to flush the lineage graph and receipts to a local manifold database.
*   **Phase 5**: Integrate `egg` for rewrite compression and e-graph equivalence searching.