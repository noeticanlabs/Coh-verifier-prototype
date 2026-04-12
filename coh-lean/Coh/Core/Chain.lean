import Coh.Prelude
import Coh.Core.Carriers
import Mathlib.Data.Real.Basic
import Mathlib.Algebra.Order.Ring.Abs

namespace Coh.Core

/-!
# Phase C: Receipt Chains (Compositional Discrete Trace Layer)

This module implements the trace-audit logic for governed systems based on the
Relative Dissipation Inequality extracted from the NotebookLM sources.

[PROVED] Compositional lawfulness with additive slack aggregation.
-/

/--
Atomic unit of a verifiable state transition.
[INVARIANT] all fields are strictly non-negative.
-/
structure Receipt where
  spend : ℝ
  slack : ℝ
  authority : ℝ
  h_spend : 0 ≤ spend := by linarith
  h_slack : 0 ≤ slack := by linarith
  h_authority : 0 ≤ authority := by linarith

/--
A CohObject is a governed wrapper around a state space V.
It encapsulates the potential functional and the current state.
-/
structure CohObject (V : Type*)
    [NormedAddCommGroup V] [NormedSpace ℝ V] [InnerProductSpace ℝ V] [CarrierSpace V] where
  state : V
  potential : V → ℝ
  budget : ℝ

/--
Lawfulness Predicate (The Relative Dissipation Inequality).
The governance law: V(x') - V(x) ≤ Slack(r) + Authority(r) - Spend(r).
This ensures that any increase in potential is strictly bounded by authorized slack.
-/
def IsLawful {V : Type*}
    [NormedAddCommGroup V] [NormedSpace ℝ V] [InnerProductSpace ℝ V] [CarrierSpace V]
    (r : Receipt) (obj obj' : CohObject V) : Prop :=
  obj'.potential obj'.state - obj.potential obj.state ≤ r.slack + r.authority - r.spend

--------------------------------------------------------------------------------
-- Chain Logic: Compositional Traces
--------------------------------------------------------------------------------

/--
Inductive type for a sequence of lawful receipts.
Forms the morphisms of the Coh Restriction Category.
-/
inductive Chain {V : Type u}
    [NormedAddCommGroup V] [NormedSpace ℝ V] [InnerProductSpace ℝ V] [CarrierSpace V] :
    CohObject V → CohObject V → Type u where
  | id (obj : CohObject V) : Chain obj obj
  | step {obj₁ obj_mid obj₂ : CohObject V} (r : Receipt) :
      IsLawful r obj₁ obj_mid → Chain obj_mid obj₂ → Chain obj₁ obj₂

/--
Aggregate two Receipts into a single effective Receipt.
[PROVED] Slack and Authority accumulate additively.
-/
def combineReceipts (r₁ r₂ : Receipt) : Receipt where
  spend := r₁.spend + r₂.spend
  slack := r₁.slack + r₂.slack
  authority := r₁.authority + r₂.authority
  h_spend := add_nonneg r₁.h_spend r₂.h_spend
  h_slack := add_nonneg r₁.h_slack r₂.h_slack
  h_authority := add_nonneg r₁.h_authority r₂.h_authority

/--
The composition of two lawful transitions satisfies the aggregate dissipation inequality.
[PROVED] via telescoping cancellation of intermediate potentials.
-/
theorem lawful_composition {V : Type*}
    [NormedAddCommGroup V] [NormedSpace ℝ V] [InnerProductSpace ℝ V] [CarrierSpace V]
    (r₁ r₂ : Receipt) (obj₁ obj₂ obj₃ : CohObject V)
    (h₁ : IsLawful r₁ obj₁ obj₂)
    (h₂ : IsLawful r₂ obj₂ obj₃) :
    IsLawful (combineReceipts r₁ r₂) obj₁ obj₃ := by
  unfold IsLawful combineReceipts at *
  -- Ensure both sides are reachable for linarith by rearranging terms
  simp only [Receipt.spend, Receipt.slack, Receipt.authority] at *
  linarith

/--
Terminal aggregation function: reduces a Chain to a single Receipt.
-/
def Chain.toReceipt {V : Type u}
    [NormedAddCommGroup V] [NormedSpace ℝ V] [InnerProductSpace ℝ V] [CarrierSpace V]
    {obj₁ obj₂ : CohObject V} (c : Chain obj₁ obj₂) : Receipt :=
  match c with
  | .id _ => { spend := 0, slack := 0, authority := 0 }
  | .step r _ next => combineReceipts r (toReceipt next)

/--
The Restriction Operator for the Coh Category.
For a trace f : obj₁ → obj₂, the restriction e(f) : obj₁ → obj₁ is the identity
on the domain, representing that the entire chain is verifiable.
-/
def Chain.restrict {V : Type u}
    [NormedAddCommGroup V] [NormedSpace ℝ V] [InnerProductSpace ℝ V] [CarrierSpace V]
    {obj₁ obj₂ : CohObject V} (_ : Chain obj₁ obj₂) : Chain obj₁ obj₁ :=
  .id obj₁

end Coh.Core
