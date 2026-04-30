#[cfg(test)]
mod tests {
    use crate::*;
    use coh_core::rv_kernel::{RvKernel, RvGoverningState, ProtectedRvBudget};
    use coh_core::types::FormalStatus;
    use coh_npe::kernel::{NpeKernel, NpeGoverningState, NpeBudget};
    use coh_phaseloom::kernel::PhaseLoomKernel;
    use coh_phaseloom::budget::PhaseLoomBudget;
    use coh_phaseloom::PhaseLoomState;
    use coh_npe::loop_engine::NpeState;
    use num_rational::Rational64 as Rational;

    fn setup_governor() -> GmiGovernor {
        let npe = NpeKernel::new(
            NpeState::new(NpeConfig::default()),
            NpeGoverningState::default(),
            NpeBudget::default(),
        );
        let rv = RvKernel::new(
            RvGoverningState::default(),
            ProtectedRvBudget::default(),
        );
        let phaseloom = PhaseLoomKernel::new(
            PhaseLoomState::default(),
            PhaseLoomBudget::default(),
        );
        let env = EnvironmentalEnvelope {
            power_mj: Some(1000),
            thermal_headroom_c: Some(20.0),
            wallclock_ms: 10000,
            hardware_available: true,
            network_allowed: true,
        };
        let system = SystemReserve {
            halt_available: true,
            logging_ops: 1000,
            ledger_append_ops: 1000,
            recovery_ops: 100,
            scheduler_ticks: 1000,
        };
        GmiGovernor::new(npe, rv, phaseloom, env, system, None)
    }

    // Helper: find any event containing a substring
    fn has_event(events: &[String], fragment: &str) -> bool {
        events.iter().any(|e| e.contains(fragment))
    }

    // ── Lean: atomic_transition_stable ──────────────────────────────────────
    #[test]
    fn test_global_law_rejects_even_if_local_kernels_pass() {
        let gov = setup_governor();
        assert!(!gov.is_globally_admissible(10, 100, 10, 0),
            "Lean: atomic_transition_stable — V(x')+spend > V(x)+defect must fail");
    }

    // ── Lean: isRationalInf_add_inf_le (budget infimum) ─────────────────────
    #[test]
    fn test_governor_blocks_budget_bleed() {
        let mut gov = setup_governor();
        gov.atom.npe.budget.cpu_ms = 0;

        let (success, trace) = gov.step("test_prop_1", "content",
            Rational::new(0, 1), Rational::new(1, 1), Rational::new(1, 1),
            FormalStatus::ProofCertified);
        assert!(!success, "Should reject when NPE budget is 0");
        // Atom emits "Atom REJECT: NPE budget exhausted" via with_reject()
        assert!(has_event(&trace.events, "NPE budget exhausted"),
            "Expected NPE budget exhaustion event, got: {:?}", trace.events);
    }

    // ── Lean: atomic_transition_rv_certified ─────────────────────────────────
    #[test]
    fn test_governor_rejects_without_receipt() {
        let mut gov = setup_governor();
        gov.atom.budgets.system.logging_ops = 0;

        let (success, _trace) = gov.step("test_prop_2", "content",
            Rational::new(0, 1), Rational::new(1, 1), Rational::new(1, 1),
            FormalStatus::ProofCertified);
        // System reserve check fires before RV gets to certify
        assert!(!success, "Should reject when logging_ops = 0");
    }

    // ── RV budget reserve protection ─────────────────────────────────────────
    #[test]
    fn test_governor_backpressure_reduces_npe_rate() {
        let mut gov = setup_governor();
        // Set RV budget too low — reserve check fires
        gov.atom.rv.budget.cpu_ms = gov.atom.rv.budget.reserve_steps_min + 5;

        let (success, trace) = gov.step("test_prop_3", "content",
            Rational::new(0, 1), Rational::new(1, 1), Rational::new(1, 1),
            FormalStatus::ProofCertified);
        assert!(!success, "Should reject when RV budget near reserve floor");
        // RV budget too low triggers Defer decision from RvKernel
        assert!(has_event(&trace.events, "RV failed"),
            "Expected RV failure event, got: {:?}", trace.events);
    }

    // ── Environmental Halt ───────────────────────────────────────────────────
    #[test]
    fn test_environmental_halt() {
        let mut gov = setup_governor();
        gov.atom.budgets.env.hardware_available = false;

        let (success, trace) = gov.step("test_prop_5", "content",
            Rational::new(0, 1), Rational::new(1, 1), Rational::new(1, 1),
            FormalStatus::ProofCertified);
        assert!(!success, "Should halt on hardware_available=false");
        assert!(has_event(&trace.events, "Env breach"),
            "Expected Env breach event, got: {:?}", trace.events);
    }

    // ── Causal Cone Enforcement ──────────────────────────────────────────────
    #[test]
    fn test_governor_enforces_causal_cone() {
        let mut gov = setup_governor();
        // Spacelike: d=2, c_g=1, dt_g=1 → d > c_g * dt_g
        let (success, trace) = gov.step("spacelike_1", "content",
            Rational::new(2, 1), Rational::new(1, 1), Rational::new(1, 1),
            FormalStatus::ProofCertified);
        assert!(!success, "Spacelike transition must be rejected");
        assert!(has_event(&trace.events, "Spacelike"),
            "Expected Spacelike rejection event, got: {:?}", trace.events);
    }

    #[test]
    fn test_spacelike_rejected_before_rv_budget_spend() {
        let mut gov = setup_governor();
        let initial_rv_cpu = gov.atom.rv.budget.cpu_ms;

        let (success, _) = gov.step("spacelike_2", "content",
            Rational::new(2, 1), Rational::new(1, 1), Rational::new(1, 1),
            FormalStatus::ProofCertified);
        assert!(!success, "Spacelike must be rejected");
        // Causal check must fire before RV budget is charged
        assert_eq!(gov.atom.rv.budget.cpu_ms, initial_rv_cpu,
            "RV budget must NOT be spent on spacelike rejection");
    }

    #[test]
    fn test_null_boundary_is_not_auto_accept() {
        let mut gov = setup_governor();
        gov.atom.npe.budget.cpu_ms = 0;

        // Null: d=1, c_g=1, dt_g=1 → lightlike, passes causal cone
        let (success, trace) = gov.step("null_1", "content",
            Rational::new(1, 1), Rational::new(1, 1), Rational::new(1, 1),
            FormalStatus::ProofCertified);
        assert!(!success, "Lightlike + zero NPE budget must still fail");
        assert!(!has_event(&trace.events, "Spacelike"),
            "Must not emit Spacelike event for lightlike transition");
        assert!(has_event(&trace.events, "NPE budget exhausted"),
            "Must emit NPE budget exhaustion, got: {:?}", trace.events);
    }

    // ── Timelike happy path ──────────────────────────────────────────────────
    #[test]
    fn test_timelike_still_requires_rv_accept() {
        let mut gov = setup_governor();
        // Timelike: d=1/2, c_g=1, dt_g=1 → interior of light cone
        let (success, trace) = gov.step("timelike_1", "content",
            Rational::new(1, 2), Rational::new(1, 1), Rational::new(1, 1),
            FormalStatus::ProofCertified);
        assert!(success, "Timelike fully-admissible transition must succeed");
        assert!(!has_event(&trace.events, "Spacelike"),
            "Timelike must not emit Spacelike rejection");
    }
}
