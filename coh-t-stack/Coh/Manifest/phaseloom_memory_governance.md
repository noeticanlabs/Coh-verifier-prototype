# PhaseLoom Memory Access Governance (Theory of Operation)

## The Core Principle
PhaseLoom is not merely a storage layer; it is a **Memory Access Governance Layer**. Access to the computational trajectory is granted based on operational need, role authority, and the required memory horizon. 

[
\boxed{
\text{access is need-based, not universal.}
}
]

## Memory Tiers
The system maintains a three-tiered hierarchy of memory to optimize for both high-frequency verification and long-term structural stability:

1. **Micro-Memory (The Fact Horizon)**:
   - Recent state transitions and receipts.
   - Immediate budget/debt updates.
   - Local instability warnings.
   - Hot path for RV/GCCP.

2. **Meso-Memory (The Pattern Horizon)**:
   - Medium-term trends and rolling windows.
   - Repeated failure motifs.
   - Compression rollups (SummaryTrajectory).
   - Context for NPE proposal bias.

3. **Macro-Memory (The Invariant Horizon)**:
   - Long-term structural invariants.
   - Calibrated operating envelopes.
   - Approved policy history and anchor sets.
   - Governance-controlled updates.

## Memory Access Policy (P_access)
Each system component is assigned a **Role** that defines its permissions over the memory tiers:

| Role | Micro Read/Write | Meso Read/Write | Macro Read/Write |
| :--- | :--- | :--- | :--- |
| **Verifier (RV)** | Read (Minimal) | Deny | Deny |
| **Admission Gate (GCCP)** | Read | Read (Summary) | Read (Policy) |
| **Generator (GMI/NPE)** | Read | Read | Read (Summary) |
| **Memory Manager (Loom)**| Read/Write | Read/Write | Read / Proposed Write |
| **Auditor** | Read | Read | Read |
| **Operator** | Read | Read | Approve / Write |

## Memory Access Views
To prevent bloat and maintain security boundaries, components do not access raw memory. Instead, they interact through **Bounded Views**:

- **AdmissionRiskView**: Provides GCCP with rejection rates, debt scores, and active policy hashes.
- **ProposalContextView**: Provides NPE with success/failure patterns and approved stable regions.
- **AuditTraceView**: Provides the full historical chain for inspection outside the hot path.

## Formal Object Representation
The extended PhaseLoom state is defined as:
[
\boxed{
\widetilde{x} = (x, C, T, B, A, \mathcal{M}, P_{\mathrm{access}})
}
]
Where:
- $\mathcal{M} = (M_{\mu}, M_{\mathrm{meso}}, M_{\mathrm{macro}})$ is the tiered memory.
- $P_{\mathrm{access}}$ is the governance policy enforcing role-based access boundaries.
