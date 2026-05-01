# Coh-Bit Governed Runtime System (Theory of Operation)

## The Coh-Bit Definition
A **Coh-bit** is a minimal governed computational unit whose state transition is valid only if verified. Unlike a classical bit which only encodes state, a Coh-bit is a **certified state-transition cell**.

### Formal Structure
A Coh-bit carries the following context:
- **Value**: The raw bit/data state.
- **State Hash**: Cryptographic commitment to the state.
- **Admissibility Status**: Boolean/Enumerated indicator of system-level validity.
- **Spend**: Resource cost consumed by the transition.
- **Defect Allowance**: Permitted exploratory/slack budget.
- **Receipt Link**: Hash link to the previous step in the causal chain.
- **Trust Context**: Signature or authority witness.

## The Transition Law
A transition $b_n \xrightarrow{r_n} b_{n+1}$ is admissible if and only if it satisfies the dual conditions of **Verification** and **Conservation**:

1. **Formal Verification (RV)**:
   $$RV(b_n, r_n, b_{n+1}) = \text{ACCEPT}$$
   The transition must be accompanied by a witness (receipt/certificate) that is valid under the system's verifier law.

2. **Resource Conservation (GCCP)**:
   $$V(b_{n+1}) + \text{Spend}(r_n) \le V(b_n) + \text{Defect}(r_n)$$
   The new state valuation ($V$) plus the process cost must not exceed the previous valuation plus the allocated slack.

## The Runtime Cycle
The Coh-bit compute cycle follows a strict pipeline:
1. **Proposal**: GMI / NPE generates a candidate transition.
2. **Admission**: GCCP checks resource availability and boundary conditions.
3. **Verification**: RV (Runtime Verifier) validates the proof certificate.
4. **Commit**: The Coh-bit updates its state and commits the hash.
5. **Receipt**: A receipt is generated and linked to the global chain.
6. **Memory**: PhaseLoom updates the trajectory history and debt calibration.

## Comparative Ontology
| Bit Type | Nature | Governing Principle |
| :--- | :--- | :--- |
| **Classical Bit** | State only | Hardware permission |
| **Qubit** | Probabilistic amplitude | Unitary evolution / Measurement |
| **Coh-Bit** | **State + Certificate + Budget** | **Verifier Law** |

## Scientific Posture
Coh bits do not compute by permission of hardware alone. They compute by passing a formal verifier law. In the context of the nuclear/fusion wedge, the Coh-bit acts as a **safety-validating transition unit**, ensuring that every control move is verified-stable before it is committed to the reactor state.
