# NPE Rust Loop Upgrade Plan: Code Patches + Dependency Upgrades

## Overview

Extend the existing NPE Rust loop from `coh-node/crates/coh-genesis/examples/npe_rust_improves_lean_loop.rs` to handle both:
1. **Code patches** (existing) - modifications to `.rs` source files
2. **Dependency upgrades** (new) - Cargo.toml edits, toolchain updates

## Architecture

```mermaid
flowchart TD
    subgraph NPE_Loop [NPE-Rust Upgrade Loop]
        A[Start] --> B{Randomly select target type}
        B -->|30%| C[CodePatch Mode]
        B -->|30%| D[Dependency Mode]  
        B -->|40%| E[Mixed Mode]
        
        subgraph CodePatch_Mode [CodePatch Mode]
            C --> C1[Select target .rs file]
            C1 --> C2[Select PatchClass<br/>Doc|Test|Strengthen|...]
            C2 --> C3[generate_patch_text]
            C3 --> C4[Create CodePatchCandidate]
            C4 --> C5[is_formation_admissible]
            C5 --> C6[Verify: cargo check/test]
        end
        
        subgraph Dependency_Mode [Dependency Mode]
            D --> D1[Parse Cargo.toml]
            D1 --> D2[Query cargo outdated]
            D2 --> D3[Select UpgradeClass<br/>VersionBump|NewCrate|Toolchain]
            D3 --> D4[generate_dep_upgrade_text]
            D4 --> D5[Create DepUpgradeCandidate]
            D5 --> D6[is_upgrade_admissible]
            D6 --> D7[Verify: cargo update --dry-run]
        end
        
        subgraph Mixed_Mode [Mixed Mode]
            E --> E1[Alternate CodePatch then DepUpgrade]
            E1 --> E2[Run code patch pipeline]
            E2 --> E3[Run dep upgrade pipeline]
            E3 --> E4[Check combined impact]
        end
        
        C6 --> F{Accept?}
        D7 --> F
        E4 --> F
        F -->|No| G[Reject & Log Failure]
        F -->|Yes| H[Apply to Project]
        H --> I[Run full cargo test]
        I --> J{Improve?}
        J -->|Yes| K[Keep Patch]
        J -->|No| L[Revert]
    end
```

## Data Structures

### New: UpgradeTarget enum
```rust
pub enum UpgradeTarget {
    CodePatch,      // .rs file modifications
    Dependency,    // Cargo.toml / toolchain updates
    Mixed,         // Both in alternating sequence
}
```

### New: DependencyUpgradeCandidate
```rust
pub struct DependencyUpgradeCandidate {
    pub id: String,
    pub wildness: f64,
    pub upgrade_class: UpgradeClass,
    pub cargo_toml_path: String,
    pub upgrade_text: String,        // TOML diff
    pub old_version: String,     // e.g., "1.2.3"
    pub new_version: String,     // e.g., "1.3.0"
    pub crate_name: String,        // e.g., "sha2"
    pub novelty: f64,
}

pub enum UpgradeClass {
    VersionBump,   // Update crate version
    NewCrate,      // Add new dependency
    RemoveCrate,   // Remove unused dependency
    Toolchain,      // Update rust-toolchain.toml
    PatchVersion,   // Bump patch for security
}
```

### New: DependencyUpgradeReport
```rust
pub struct DependencyUpgradeReport {
    pub cargo_update_dry_run_pass: bool,
    pub cargo_check_pass: bool,
    pub outdated_check_pass: bool,
    pubbreaking_change: bool,
    pub new_version_available: bool,
    pub update_size_kb: u64,
    pub upgrade_margin: i128,
}
```

## Key Functions to Add

### 1. parse_cargo_toml(path: &str) -> HashMap<String, String>
Parse Cargo.toml and extract current dependencies with versions.

### 2. check_cargo_outdated(path: &str) -> Vec<CrateUpdate>
Run `cargo outdated --format json` and parse available updates.

### 3. generate_dep_upgrade_text(class: UpgradeClass, crate: &str, from: &str, to: &str) -> String
Generate TOML diff for dependency changes.

### 4. is_upgrade_admissible(candidate: &DepUpgradeCandidate, report: &DepUpgradeReport) -> (bool, i128)
Validate upgrade using genesis/coherence margins.

## Implementation Plan

| Step | File | Task |
|------|------|------|
| 1 | `code_patch.rs` | Add `UpgradeTarget`, `UpgradeClass`, `DependencyUpgradeCandidate`, `DependencyUpgradeReport` |
| 2 | `code_patch.rs` | Add `parse_cargo_toml()`, `check_cargo_outdated()` |
| 3 | `code_patch.rs` | Add `generate_dep_upgrade_text()`, `is_upgrade_admissible()` |
| 4 | `code_patch.rs` | Update `build_formation_result()` to handle both types |
| 5 | New example | Create `npe_rust_upgrade_loop.rs` demonstrating both modes |

## Validation Rules

### Genesis Margin for Dep Upgrades
```
genesis_margin = base_complexity + defect_budget - update_size_kb * 10 - audit_cost
```

where:
- `base_complexity`: 500 (workspace baseline)
- `defect_budget`: 100 + wildness * 20
- `update_size_kb`: Size of update in KB
- `audit_cost`: Estimated review time

### Coherence Margin for Dep Upgrades
```
coherence_margin = risk_tolerance + def_budget - breaking_risk - integration_cost
```

where:
- `risk_tolerance`: 200 (default)
- `def_budget`: 50
- `breaking_risk`: 0 if semver compatible, 100 otherwise
- `integration_cost`: 10 * number of dependent crates

### Hard Gates
```rust
// Reject if:
- breaking_change && !allow_breaking  
- update_size_kb > max_update_size_kb
- crate_name.contains("unsafe") || crate_name.contains("yolo")
- toolchain.version < current_min version
```

## Example Usage

```rust
// CodePatch mode
let code_patch = CodePatchCandidate {
    id: "patch-001".to_string(),
    wildness: 0.3,
    target_file: "semantic.rs".to_string(),
    patch_text: "...".to_string(),
    // ...
};

// Dependency mode
let dep_upgrade = DependencyUpgradeCandidate {
    id: "upgrade-001".to_string(),
    wildness: 0.2,
    upgrade_class: UpgradeClass::VersionBump,
    cargo_toml_path: "coh-node/crates/coh-core/Cargo.toml".to_string(),
    upgrade_text: "sha2 = \"0.2\" → sha2 = \"0.3\"".to_string(),
    old_version: "0.2".to_string(),
    new_version: "0.3".to_string(),
    crate_name: "sha2".to_string(),
    novelty: 0.4,
};

// Verify both through their respective admissibility checks
let (code_accept, code_margin) = is_formation_admissible(&code_patch, base, &code_report);
let (dep_accept, dep_margin) = is_upgrade_admissible(&dep_upgrade, base, &dep_report);
```

## Files to Modify

1. **Add to**: `coh-node/crates/coh-genesis/src/code_patch.rs`
   - New enums and structs (400+ lines)
   - New functions (200+ lines)

2. **Create**: `coh-node/crates/coh-genesis/examples/npe_rust_upgrade_loop.rs`
   - Full example demonstrating both modes (200+ lines)

## Success Criteria

- [ ] Generates valid CodePatchCandidates (existing)
- [ ] Generates valid DependencyUpgradeCandidates
- [ ] Parses Cargo.toml correctly
- [ ] Runs cargo outdated --format json
- [ ] Generates valid TOML diffs
- [ ] Validates with is_upgrade_admissible()
- [ ] Alternates between modes in Mixed mode
- [ ] Tracks improvement metrics for both types