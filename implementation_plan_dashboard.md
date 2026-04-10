# Implementation Plan - Coh Integrity Dashboard

Build a high-rigor, premium "Integrity Dashboard" to visualize the Coh state-transition chains. This UI transforms the abstract "No-Bluff" protocol into a tangible audit experience.

![Integrity Dashboard Mockup](file:///C:/Users/truea/.gemini/antigravity/brain/fed755d2-1cd5-4ca1-b817-01babe6662db/coh_integrity_dashboard_mockup_1775860400973.png)

## User Review Required

> [!IMPORTANT]
> **WASM vs API Architecture**: For Phase 1, the UI will be a standalone dashboard using mocked verification logic that mirrors the Rust kernel. In Phase 2, should we prioritize a **local Rust sidecar (HTTP)** or a **direct WASM build** of coh-core for the browser?
>
> **Data Format**: I will assume the `.jsonl` format used by the CLI is the primary input for the dashboard's "Verify Chain" feature.

## Proposed Changes

### [NEW] [coh-dashboard](file:///c:/Users/truea/.gemini/antigravity/brain/75ab5e4c-a69b-451c-b975-f05bf7a3baf3/Coh%20wedge/coh-dashboard)

A new React application bootstrapped with Vite.

#### Architecture
- **State Management**: React Context for the active receipt chain and verification results.
- **Styling**: Vanilla CSS with a global `theme.css` for the "Vercel-esque" neon/dark-mode aesthetics.
- **Component Stack**:
    - `Header.jsx`: Title, global status (TRUSTED/TAMPERED).
    - `ChainVisualizer.jsx`: A horizontal, scrollable flow of nodes (green/red).
    - `ReceiptDetail.jsx`: A side panel showing the raw JSON and the "Accounting Law" math ($v_{post} + spend \le v_{pre} + defect$).
    - `VerifiedTimeline.jsx`: A vertical list of transitions with time-independent step indices and hashes.

#### [NEW] [index.html](file:///c:/Users/truea/.gemini/antigravity/brain/75ab5e4c-a69b-451c-b975-f05bf7a3baf3/Coh%20wedge/coh-dashboard/index.html)
- SEO tags for "Coh State Integrity Auditor".
- Google Font integration (Inter/Outfit).

## Verification Plan

### Automated Tests
1. **Frontend Verify**: Run `npm run build` to ensure no lint/build errors.
2. **Chain Simulation**: Test the dashboard with a "broken" receipt chain (invalid state_hash_prev link) to ensure the UI correctly highlights the breach node in RED.

### Manual Verification
- Verify the "Wow" factor: Neon glow effects on valid links, pulsing red alerts on tampered steps.
- Test responsive layout for tablet-sized monitoring screens.
