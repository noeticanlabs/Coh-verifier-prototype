use coh_core::vm::Runtime;
use coh_core::cohbit::{CohBit};
use coh_core::atom::CohAtom;
use coh_core::spinor::CohSpinor;
use coh_core::types::{Hash32, DomainId, RvStatus};
use coh_phaseloom::{PhaseLoomState, PhaseLoomConfig};
use coh_npe::{NpeEngine, NpeConfig};
use num_rational::Rational64;
use rand::thread_rng;

/// Mock Generator for the VM demonstration
struct MockGenerator;
impl coh_npe::traits::NpeGenerator for MockGenerator {
    type Context = ();
    fn generate(&self, _seed: u64, _index: usize, _ctx: &Self::Context) -> Result<String, coh_npe::engine::NpeError> {
        Ok("mock_proposal".to_string())
    }
}

/// PhaseLoomRuntime: Bridging the Phase-Structured Memory Manifold with CohVM.
pub struct PhaseLoomRuntime {
    pub loom: PhaseLoomState,
    pub npe: NpeEngine,
    pub config: PhaseLoomConfig,
    pub domain: DomainId,
}

impl PhaseLoomRuntime {
    pub fn new(domain: DomainId, npe_config: NpeConfig) -> Self {
        let pl_config = PhaseLoomConfig::default();
        Self {
            loom: PhaseLoomState::new(&pl_config),
            npe: NpeEngine::new(npe_config),
            config: pl_config,
            domain,
        }
    }
}

impl Runtime for PhaseLoomRuntime {
    fn propose_candidates(&self, state: Hash32) -> Vec<CohBit> {
        let mut _rng = thread_rng();
        
        // 1. Generate proposals from NPE
        let generator = MockGenerator;
        let proposals = self.npe.generate_proposals(10, &generator, &()).unwrap_or_default();

        // 2. Convert NpeProposals to CohBits
        proposals.into_iter().enumerate().map(|(i, _p)| {
            let mut bit = CohBit::identity_atom(state, Rational64::from_integer(100), self.domain);
            // fixture_only: allow_mock
            let mut id = [0xEE; 32];
            let rand_val = rand::random::<u64>();
            id[0..8].copy_from_slice(&rand_val.to_be_bytes());
            id[8..10].copy_from_slice(&(i as u16).to_be_bytes());
            bit.bit_id = Hash32(id);
            bit.action_hash = Hash32([0xDD; 32]); 
            bit.to_state = bit.action_hash; 
            bit.signature = coh_core::types::Signature(vec![0xFF; 64]); // fixture_only: allow_mock
            bit.valuation_pre = Rational64::from_integer(10000);
            bit.valuation_post = Rational64::from_integer(9995);
            bit.spend = Rational64::from_integer(5);
            bit.defect = Rational64::from_integer(5);
            bit.delta_hat = Rational64::from_integer(15);
            bit.utility = Rational64::from_integer(1);
            bit.rv_status = RvStatus::Accept;
            bit.finalize_hashes()
        }).collect()
    }

    fn retrieve_from_memory(&self, spinor: &CohSpinor) -> Vec<CohAtom> {
        let epsilon = Rational64::new(72, 100);
        let max_hits = 8;
        let radius = 1;
        self.loom.retrieve(spinor, epsilon, max_hits, radius).into_iter().cloned().collect()
    }

    fn weave_to_memory(&mut self, atom: CohAtom, final_spinor: &CohSpinor) {
        self.loom.weave(atom, final_spinor);
    }

    fn execute(&self, action: Hash32) -> Hash32 {
        action
    }
}
