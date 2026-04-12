# Coh Safety Wedge — Formal Foundation

The Coh system is anchored by the **T-Stack Federated Ledger**, a multi-phase machine-verified proof system.

---

## The T-Stack Ledger (Verified T1–T5)

| Phase | Theorem | Verified Pillar | Formal Source |
|-------|---------|-----------------|---------------|
| **A** | **T1** | Categorical Extraction | `Coh.Kernel.T1_Category` |
| **B** | **T2** | Oplax Slack Bridge | `Coh.Slack.T2_OplaxBridge` |
| **C** | **T3** | Macro-Slab Aggregate | `Coh.Trace.T3_MacroSlab` |
| **D** | **T4** | Anomaly Visibility | `Coh.Spectral.T4_Visibility` |
| **E** | **T5** | Dirac Representation | `Coh.Selection.T5_DiracSelection` |

---

## Core Invariants

### 1. The Accounting Law (Kernel Level)

The absolute discrete verifier enforces:
```
v_post + spend <= v_pre + defect + authority
```
**Decision Logic**: `ACCEPT` if the inequality holds, otherwise `REJECT`.
**Source**: [Coh/Kernel/Verifier.lean](coh-t-stack/Coh/Kernel/Verifier.lean)

### 2. Oplax Preservation (Slack Layer)

Finite precision and coarse-graining are managed via the **Oplax Injection**, proving that:
`LawfulUpTo ? ? Lawful (paySlack ?)`
This allows the stack to maintain global safety while internalizing operational slack.
**Source**: [Coh/Slack/T2_OplaxBridge.lean](coh-t-stack/Coh/Slack/T2_OplaxBridge.lean)

### 3. Trace Homomorphism (Macro Layer)

Individual micro-transitions are algebraically aggregatable into macro-slabs via telescoping sums, preserving the integrity of the total chain trace.
**Source**: [Coh/Trace/T3_MacroSlab.lean](coh-t-stack/Coh/Trace/T3_MacroSlab.lean)

---

## Physical Necessity (Spectral Wedge)

The **Visibility Theorem (T4)** guarantees that any broken symmetry or adversarial tamper produces a strictly observable anomaly ($|\Delta| \ge \varepsilon$), ensuring that the verifier cannot be bypassed by infinitesimal violations.

The **Dirac Selection Theorem (T5)** proves that the Dirac spinor is the unique minimal faithful metabolic carrier, grounding the security stack in thermodynamic necessity.
