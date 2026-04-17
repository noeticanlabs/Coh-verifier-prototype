# APE Simplified Plan - QuickCheck for Investor Demo

## Current Issue
APE as built: **too complex for a quick investor demo**
- 5 strategy modules (mutation, recombination, violation, overflow, contradiction)
- Multiple layers of data structures (Input, Proposal, Candidate)
- Custom seeded RNG

## User Direction
> "Stop overengineering — use QuickCheck as proposal generator"

## Simplified Design

### Core API (Minimal)
```rust
// Simple proposal struct - no complex strategies needed
struct Proposal {
    state: State,
    action: Action,
    is_adversarial: bool,  // Controls valid vs invalid
}

// QuickCheck handles randomness + determinism
// APE handles structure + intent
```

### Demo Flow (3 Steps)
```
Step 1: QuickCheck generates N proposals (valid + adversarial)
Step 2: Each proposal → Verifier
Step 3: Display Pass/Fail results
```

## Implementation

### Step 1: Add quickcheck to Cargo.toml
```toml
[dependencies]
quickcheck = "1.0"
```

### Step 2: Simple Proposal struct
```rust
#[derive(Clone, Debug, Arbitrary)]
pub struct Proposal {
    pub state: u128,
    pub action: Action,
    pub is_adversarial: bool,
}

#[derive(Clone, Debug, Arbitrary)]
pub enum Action {
    Transfer { amount: u128, target: String },
    // etc.
}
```

### Step 3: Implement Arbitrary for Proposal
```rust
impl Arbitrary for Proposal {
    fn arbitrary(g: &mut Gen) -> Self {
        // QuickCheck handles randomness
        // Generate valid + adversarial cases
    }
}
```

### Step 4: Demo Command
```rust
fn demo() {
    // QuickCheck::new() with seed for determinism
    let mut gen = QuickCheck::new();
    
    for _ in 0..100 {
        let prop = gen.generate();
        let result = verify(&prop);
        print!("{} -> {}", prop, result);
    }
}
```

## Why This Works for Demo

| Aspect | Value |
|--------|-------|
| Footprint | Tiny (quickcheck crate only) |
| Determinism | Via Gen seed |
| Fast to wire | Hours not days |
| Investor pitch | "We generate bad inputs automatically, verifier catches everything" |

## Migration Path (Future)
- Phase 2: Switch to proptest for better control
- Keep current as production APE

## Plan for Implementation

1. [ ] Add quickcheck to ape/Cargo.toml
2. [ ] Create simple Proposal + Action structs  
3. [ ] Implement Arbitrary for proposal generation
4. [ ] Simplify main.rs demo to use QuickCheck
5. [ ] Run demo proving valid → pass, invalid → fail

## Demo Pitch

> "We generate N adversarial transitions automatically using QuickCheck..."
> "...and the verifier catches every invalid one in real time."

That's it. No one asks about wedge manifolds after that.