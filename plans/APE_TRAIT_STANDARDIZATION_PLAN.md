# APE Trait Standardization Plan (Updated)

## Decision
Keep the 5-strategy design (Mutation, Recombination, Violation, Overflow, Contradiction) for the investor demo.

## Architecture

```
┌─────────────────────────────┐
│         APE Engine           │
├─────────────────────────────┤
│ Strategy Enum (enum dispatch)│
│ + generate() method         │
└──────────┬────────────────┬┘
           │                │
    ┌──────┴─────┐  ┌───────┴────┐
    │            │  │            │
    ▼            ▼  ▼            ▼
Mutation   Recombination  Violation
    │            │            │
    ▼            ▼            ▼
Overflow  Contradiction       ...
    │            │
    └──────┬─────┘
           ▼
    SeededRng
           ▼
    Candidate → Verifier
```

## Implementation

### Step 1: Define Candidate with metadata
The Candidate must carry enough info to explain itself:

```rust
pub struct Candidate {
    pub strategy_name: &'static str,      // "mutation", "overflow", etc.
    pub attack_kind: &'static str,        // specific corruption type
    pub seed: u64,                       // for replay
    pub notes: String,                    // human-readable explanation
    pub receipt: MicroReceiptWire,         // the actual receipt
}
```

### Step 2: Strategy enum with direct dispatch
No trait objects needed for 5 fixed strategies:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    Mutation,
    Recombination,
    Violation,
    Overflow,
    Contradiction,
}

impl Strategy {
    pub fn name(&self) -> &'static str { ... }
    pub fn note(&self) -> &'static str { ... }  // for narrative
    pub fn generate(&self, input: &Input, rng: &mut SeededRng) -> Candidate { ... }
}
```

### Step 3: Per-strategy demo with extended metrics
Track more than just rejection counts:

```rust
fn run_strategy_demo(strategy: Strategy, iterations: usize, input: &Input) -> StrategyMetrics {
    let mut accepted = 0;
    let mut rejected = 0;
    let mut escaped = 0;
    let mut first_escaped_seed = None;
    let mut latencies = Vec::new();
    
    for seed in 0..iterations {
        let mut rng = SeededRng::new(seed);
        let start = Instant::now();
        let candidate = strategy.generate(input, &mut rng);
        let result = verify_micro(&candidate.receipt);
        let elapsed = start.elapsed().as_micros() as u64;
        latencies.push(elapsed);
        
        match result.decision {
            Decision::Accept => {
                escaped += 1;
                if first_escaped_seed.is_none() {
                    first_escaped_seed = Some(seed);
                }
            }
            Decision::Reject => rejected += 1,
        }
    }
    
    StrategyMetrics {
        strategy: strategy.name(),
        iteration_note: strategy.note(),
        generated: iterations,
        rejected,
        escaped,
        first_escaped_seed,
        avg_latency_us: avg(&latencies),
        worst_latency_us: latencies.iter().max(),
    }
}
```

### Step 4: Demo table output

| Strategy      | Generated | Rejected | Escaped | Notes                    |
| ------------- | --------: | -------: | ------: | ------------------------ |
| Mutation      |       100 |      100 |       0 | receipt field corruption |
| Recombination |       100 |      100 |       0 | chain splice attempt     |
| Violation    |       100 |      100 |       0 | invariant breach         |
| Overflow     |       100 |      100 |       0 | bounds stress            |
| Contradiction |       100 |      100 |       0 | inconsistent claims      |

### Step 5: Investor pitch (safe framing)

> "APE generates five semantic classes of adversarial proposals—mutation, recombination, violation, overflow, and contradiction—and the verifier reports deterministic rejection performance for each class with bounded latency."

## Plan

1. [ ] Update `Candidate` struct to include metadata fields
2. [ ] Refactor `Strategy` enum with `generate()` and `note()` methods
3. [ ] Update each strategy module to populate metadata
4. [ ] Implement per-strategy demo with latency tracking
5. [ ] Run demo and capture real rejection rates
6. [ ] Update investor pitch to use safe framing

## Investor Pitch

> "We test five semantic classes of adversarial receipt corruption and report verifier rejection performance by attack class, with deterministic replay for every case. Avg latency: <X>µs, Worst: <Y>µs"

That's cleaner. Less chest-thumping, more credibility.