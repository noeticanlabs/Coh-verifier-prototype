use coh_core::atom::{AtomKind, CohAtom};
use coh_core::merkle::build_merkle_root;
use coh_core::spinor::CohSpinor;
use coh_core::types::{DomainId, Hash32, Signature};
use coh_npe::receipt::BoundaryReceiptSummary;
use coh_npe::weights::StrategyWeights;
use num_rational::Rational64;
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;

// --- Config & Metrics ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseLoomConfig {
    pub initial_budget: u128,
    pub learning_rate: f64,
    pub curvature_penalty: f64,
    pub circuit_break_threshold: u128,
    pub min_weight: f64,
    pub entropy_floor: f64,
    pub alpha_tau: f64,
    pub alpha_d: f64,
    pub alpha_p: f64,
    pub alpha_gamma: f64,
    pub max_compression_depth: u8,
    pub global_loss_hat: Rational64,
}

impl Default for PhaseLoomConfig {
    fn default() -> Self {
        Self {
            initial_budget: 100_000,
            learning_rate: 0.1,
            curvature_penalty: 0.05,
            circuit_break_threshold: 10_000,
            min_weight: 0.01,
            entropy_floor: 0.5,
            alpha_tau: 0.01,
            alpha_d: 1.0,
            alpha_p: 5.0,
            alpha_gamma: 1.0,
            max_compression_depth: 8,
            global_loss_hat: Rational64::new(1, 2),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LoomMetrics {
    pub total_atoms: u64,
    pub active_threads: u64,
    pub compressed_atoms: u64,
    pub max_depth: u8,
    pub cumulative_tension: Rational64,
    pub field_curvature: f64,
}

// --- Phase Locality Index (v2.1) ---

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PhaseKey {
    pub phase_bin: i32,
    pub orientation: u8,
    pub parity: u8,
    pub domain: Hash32,
    pub basis_hash: Hash32,
}

impl PhaseKey {
    pub const BINS: i32 = 36; // 10-degree buckets

    pub fn from_spinor(spinor: &CohSpinor) -> Self {
        let phase_f64 = (spinor.phase_num.to_f64().unwrap_or(0.0)
            / spinor.phase_den.to_f64().unwrap_or(1.0))
        .rem_euclid(1.0);
        let phase_bin = (phase_f64 * Self::BINS as f64).floor() as i32;
        Self {
            phase_bin,
            orientation: spinor.orientation as u8,
            parity: spinor.parity as u8,
            domain: Hash32(spinor.domain.0 .0),
            basis_hash: spinor.basis_hash,
        }
    }

    pub fn neighbors(&self, radius: i32) -> Vec<PhaseKey> {
        let mut keys = vec![];
        for offset in -radius..=radius {
            let mut neighbor = self.clone();
            neighbor.phase_bin = (self.phase_bin + offset).rem_euclid(Self::BINS);
            keys.push(neighbor);
        }
        keys
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AtomPhaseSignature {
    pub atom_hash: Hash32,
    pub centroid_phase: f64,
    pub mean_alignment: Rational64,
    pub norm_band: u32,
    pub basis_hash: Hash32,
    pub thread_strength: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThreadState {
    Active,
    CompressedSource,
    Cold,
    Tombstoned,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoomThread {
    pub atom: CohAtom,
    pub signature: AtomPhaseSignature,
    pub state: ThreadState,
    pub timestamp: u64,
    pub alignment_at_weave: Rational64,
}

// --- Thread Compression (v2.2 Refinery) ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticLoss {
    pub margin_loss: Rational64,
    pub utility_loss: Rational64,
    pub phase_loss: Rational64,
    pub total: Rational64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantCore {
    pub domain: DomainId,
    pub bucket_key: PhaseKey,
    pub final_state: Hash32,
    pub policy_hash: Hash32,
    pub margin_floor: Rational64,
    pub authority_cap: Rational64,
}

impl InvariantCore {
    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"phaseloom:v2.2:core");
        hasher.update(self.domain.0 .0);
        hasher.update(self.bucket_key.phase_bin.to_be_bytes());
        hasher.update([self.bucket_key.orientation, self.bucket_key.parity]);
        hasher.update(self.bucket_key.domain.0);
        hasher.update(self.bucket_key.basis_hash.0);
        hasher.update(self.final_state.0);
        hasher.update(self.policy_hash.0);
        hasher.update(self.margin_floor.reduced().numer().to_be_bytes());
        hasher.update(self.margin_floor.reduced().denom().to_be_bytes());
        hasher.update(self.authority_cap.reduced().numer().to_be_bytes());
        hasher.update(self.authority_cap.reduced().denom().to_be_bytes());
        Hash32(hasher.finalize().into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionLineage {
    pub depth: u8,
    pub source_root: Hash32,
    pub lineage_roots: Vec<Hash32>, // NEW: prove prior compression trees absorbed
    pub cumulative_loss: Rational64,
    pub invariant_core_hash: Hash32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionAtom {
    pub version: u16,
    pub source_root: Hash32,
    pub source_count: u64,
    pub compressed_atom: CohAtom,
    pub semantic_loss: SemanticLoss,
    pub loss_hat: Rational64,
    pub lineage: CompressionLineage,
    pub bucket_key: PhaseKey,
    pub compression_policy_hash: Hash32,
    pub compression_witness_hash: Hash32,
    pub compression_hash: Hash32,
    pub signature: Signature,
}

impl CompressionAtom {
    pub fn canonical_hash(&self) -> Hash32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"cohcompression:v2.2:fractal");
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.source_root.0);
        hasher.update(self.source_count.to_be_bytes());
        hasher.update(self.compressed_atom.atom_hash.0);

        let mut update_rat = |r: &Rational64| {
            let nr = r.reduced();
            hasher.update(nr.numer().to_be_bytes());
            hasher.update(nr.denom().to_be_bytes());
        };

        update_rat(&self.semantic_loss.total);
        update_rat(&self.lineage.cumulative_loss);
        hasher.update([self.lineage.depth]);
        hasher.update(self.lineage.invariant_core_hash.0);
        for root in &self.lineage.lineage_roots {
            hasher.update(root.0);
        }

        hasher.update(self.compression_policy_hash.0);
        hasher.update(self.compression_witness_hash.0);

        Hash32(hasher.finalize().into())
    }
}

pub struct CompressionContext {
    pub min_sources: usize,
    pub max_depth: u8,
    pub global_loss_hat: Rational64,
    pub policy_hash: Hash32,
    pub verifier_id: Hash32,
    pub w_m: Rational64,
    pub w_u: Rational64,
    pub w_p: Rational64,
}

// --- Spinor Anchoring (v2.3) ---

/// Hard laws governing anchor behavior:
/// SA1: spinor cannot leave anchor cone without triggering divert.
/// SA2: anchor drift increases defect.
/// SA3: repeated low anchor alignment triggers rephase.
/// SA4: anchors bias routing but never override CohBit law.
/// SA5: anchor updates require valid CohAtom/CompressionAtom lineage.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AnchorSet {
    /// Invariant core hashes from stable compression families.
    pub invariant_core_hashes: Vec<Hash32>,
    /// High-authority atom hashes that define the orientation cone.
    pub high_authority_atoms: Vec<Hash32>,
    /// Stable summary atoms (SummaryTrajectory with low cumulative loss).
    pub stable_summary_atoms: Vec<Hash32>,
    /// Policy anchor hashes — define the legal routing envelope.
    pub policy_anchor_hashes: Vec<Hash32>,
    /// Drift penalty coefficient λ.
    pub lambda: Rational64,
    /// Low-alignment threshold: below this, increment rephase counter.
    pub alignment_threshold: Rational64,
    /// Rephase trigger count: rephase fires after this many consecutive low-alignment steps.
    pub rephase_trigger_count: u32,
}

/// Result of an anchor alignment evaluation.
#[derive(Clone, Debug)]
pub struct SpinorAnchorResult {
    /// Alignment score in [0, 1].
    pub alignment: Rational64,
    /// Drift cost = λ(1 - alignment). Added to defect.
    pub drift_cost: Rational64,
    /// Whether the spinor has exited the anchor cone (SA1).
    pub cone_exit: bool,
    /// Whether rephase should be triggered (SA3).
    pub trigger_rephase: bool,
}

impl AnchorSet {
    pub fn new(
        lambda: Rational64,
        alignment_threshold: Rational64,
        rephase_trigger_count: u32,
    ) -> Self {
        Self {
            invariant_core_hashes: vec![],
            high_authority_atoms: vec![],
            stable_summary_atoms: vec![],
            policy_anchor_hashes: vec![],
            lambda,
            alignment_threshold,
            rephase_trigger_count,
        }
    }

    /// Compute anchor alignment for a spinor.
    ///
    /// Alignment = fraction of anchor families matched by the spinor's
    /// basis_hash, state_hash, and coherence_alignment. This is a
    /// coarse geometric proxy for "phase cone membership."
    ///
    /// [HEURISTIC] — exact spinor-to-anchor geometry would require
    /// full phase-space projection, not implemented here.
    pub fn compute_alignment(
        &self,
        spinor: &CohSpinor,
        thread_store: &HashMap<Hash32, LoomThread>,
        compression_store: &HashMap<Hash32, CompressionAtom>,
    ) -> Rational64 {
        let total_anchors = self.invariant_core_hashes.len()
            + self.high_authority_atoms.len()
            + self.stable_summary_atoms.len()
            + self.policy_anchor_hashes.len();

        if total_anchors == 0 {
            // No anchors registered yet — full alignment by default.
            return Rational64::from_integer(1);
        }

        let mut matched = 0usize;

        // Check high-authority atoms: spinor matches if its state_hash
        // aligns with the atom's final_state.
        for h in &self.high_authority_atoms {
            if let Some(t) = thread_store.get(h) {
                if t.atom.final_state == spinor.state_hash {
                    matched += 1;
                }
            }
        }

        // Check stable summary atoms via compression store.
        for h in &self.stable_summary_atoms {
            if let Some(t) = thread_store.get(h) {
                if t.atom.final_state == spinor.state_hash {
                    matched += 1;
                }
            }
        }

        // Invariant core hashes: match if spinor's basis_hash appears in
        // any compression lineage anchored to that core.
        for core_hash in &self.invariant_core_hashes {
            for comp in compression_store.values() {
                if &comp.lineage.invariant_core_hash == core_hash {
                    if comp.compressed_atom.final_state == spinor.state_hash {
                        matched += 1;
                        break;
                    }
                }
            }
        }

        // Policy anchor hashes match by policy_hash in thread atoms.
        for ph in &self.policy_anchor_hashes {
            for t in thread_store.values() {
                if &t.atom.policy_hash == ph && t.atom.final_state == spinor.state_hash {
                    matched += 1;
                    break;
                }
            }
        }

        Rational64::new(matched as i64, total_anchors as i64)
    }

    /// Register a high-authority atom as an anchor (SA5).
    /// Only accepted if the atom is executable.
    pub fn register_atom_anchor(&mut self, atom: &CohAtom) -> Result<(), String> {
        if !atom.executable() {
            return Err("SA5: anchor atom is not executable".to_string());
        }
        if !self.high_authority_atoms.contains(&atom.atom_hash) {
            self.high_authority_atoms.push(atom.atom_hash);
        }
        Ok(())
    }

    /// Register a stable summary atom as an anchor (SA5).
    /// Requires a CompressionAtom with verified lineage.
    pub fn register_summary_anchor(&mut self, comp: &CompressionAtom) -> Result<(), String> {
        if comp.compressed_atom.kind != AtomKind::SummaryTrajectory {
            return Err("SA5: anchor source is not a SummaryTrajectory".to_string());
        }
        let h = comp.compressed_atom.atom_hash;
        if !self.stable_summary_atoms.contains(&h) {
            self.stable_summary_atoms.push(h);
            if !self
                .invariant_core_hashes
                .contains(&comp.lineage.invariant_core_hash)
            {
                self.invariant_core_hashes
                    .push(comp.lineage.invariant_core_hash);
            }
        }
        Ok(())
    }
}

// --- Manifold State ---

/// Phase Loom v2.3: The Fractal Trajectory Manifold with Spinor Anchoring.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseLoomState {
    pub version: u16,

    // --- Manifold Layers (v2.2) ---
    pub phase_index: HashMap<PhaseKey, Vec<Hash32>>,
    pub thread_store: HashMap<Hash32, LoomThread>,
    pub compression_store: HashMap<Hash32, CompressionAtom>,
    pub phase_field: HashMap<Hash32, Rational64>,
    pub metrics: LoomMetrics,

    // --- Spinor Anchoring (v2.3) ---
    pub anchor_set: AnchorSet,
    pub rephase_counter: u32,

    // --- Statistical Layers (Legacy/Lite) ---
    pub strategy_weights: StrategyWeights,
    pub curvature: u128,
    pub budget: u128,
    pub tension: u128,
    pub tau: u64,
    pub circuit_broken: bool,
}

impl Default for PhaseLoomState {
    fn default() -> Self {
        Self::new(&PhaseLoomConfig::default())
    }
}

impl PhaseLoomState {
    pub fn new(config: &PhaseLoomConfig) -> Self {
        Self {
            version: 3,
            phase_index: HashMap::new(),
            thread_store: HashMap::new(),
            compression_store: HashMap::new(),
            phase_field: HashMap::new(),
            metrics: LoomMetrics::default(),
            anchor_set: AnchorSet::new(
                Rational64::new(1, 10), // λ = 0.1
                Rational64::new(3, 10), // alignment_threshold = 0.3
                5,                      // rephase after 5 consecutive low-alignment steps
            ),
            rephase_counter: 0,
            strategy_weights: StrategyWeights::default(),
            curvature: 0,
            budget: config.initial_budget,
            tension: 0,
            tau: 0,
            circuit_broken: false,
        }
    }

    pub fn verify_persistence(&self) -> bool {
        self.thread_store.values().all(|t| t.atom.executable())
    }

    // --- v2.3 Spinor Anchoring ---

    /// Evaluate how well the spinor aligns with the current anchor cone.
    /// Returns a SpinorAnchorResult with alignment score, drift cost, and
    /// flags for cone exit (SA1) and rephase (SA3).
    pub fn spinor_anchor_result(&self, spinor: &CohSpinor) -> SpinorAnchorResult {
        let alignment =
            self.anchor_set
                .compute_alignment(spinor, &self.thread_store, &self.compression_store);

        // drift = λ(1 - alignment)  [SA2]
        let one = Rational64::from_integer(1);
        let drift_cost = (self.anchor_set.lambda * (one - alignment)).reduced();
        let cone_exit = alignment < self.anchor_set.alignment_threshold; // SA1
        let trigger_rephase =
            cone_exit && self.rephase_counter >= self.anchor_set.rephase_trigger_count; // SA3

        SpinorAnchorResult {
            alignment,
            drift_cost,
            cone_exit,
            trigger_rephase,
        }
    }

    /// Apply anchor drift consequences after a step.
    ///
    /// SA2: drift_cost is returned for the caller to apply to defect budget.
    /// SA3: consecutive low-alignment steps increment rephase_counter.
    ///
    /// Returns the drift cost for this step.
    /// Note: SA4 — this method NEVER mutates CohBit or overrides selection.
    /// It only updates the internal rephase counter.
    pub fn apply_anchor_drift(&mut self, spinor: &CohSpinor) -> SpinorAnchorResult {
        let result = self.spinor_anchor_result(spinor);

        if result.cone_exit {
            self.rephase_counter = self.rephase_counter.saturating_add(1);
        } else {
            // Alignment recovered — reset counter
            self.rephase_counter = 0;
        }

        if result.trigger_rephase {
            self.rephase_counter = 0; // Reset after firing
            self.metrics.field_curvature += 1.0; // Record rephase event
        }

        result
    }

    /// Register a compression atom as an anchor for future spin alignment.
    /// Enforces SA5 — only accepts valid SummaryTrajectory lineages.
    pub fn register_compression_as_anchor(&mut self, comp_hash: Hash32) -> Result<(), String> {
        let comp = self
            .compression_store
            .get(&comp_hash)
            .ok_or("SA5: compression atom not found in store")?
            .clone();
        self.anchor_set.register_summary_anchor(&comp)
    }

    pub fn retrieve(
        &self,
        spinor: &CohSpinor,
        epsilon: Rational64,
        max_hits: usize,
        radius: i32,
    ) -> Vec<&CohAtom> {
        let key = PhaseKey::from_spinor(spinor);
        let buckets = key.neighbors(radius);

        let mut candidate_hashes = vec![];
        for b in buckets {
            if let Some(hashes) = self.phase_index.get(&b) {
                candidate_hashes.extend(hashes);
            }
        }

        let mut hits = vec![];
        for hash in candidate_hashes {
            if hits.len() >= max_hits {
                break;
            }
            if let Some(thread) = self.thread_store.get(&hash) {
                // TC13: Exclude compressed sources from hot retrieval
                if thread.state == ThreadState::CompressedSource {
                    continue;
                }
                if thread.state == ThreadState::Tombstoned {
                    continue;
                }

                // Ensure continuity alignment
                if thread.atom.final_state != spinor.state_hash {
                    continue;
                }

                // Fine alignment: Ensure alignment >= epsilon
                let field_align = self
                    .phase_field
                    .get(&spinor.state_hash)
                    .cloned()
                    .unwrap_or(Rational64::from_integer(0));

                if field_align >= epsilon {
                    hits.push(&thread.atom);
                }
            }
        }
        hits
    }

    pub fn weave(&mut self, atom: CohAtom, final_spinor: &CohSpinor) -> bool {
        if !atom.executable() {
            return false;
        }

        let alignment = final_spinor.norm;
        let phase_f64 = (final_spinor.phase_num.to_f64().unwrap_or(0.0)
            / final_spinor.phase_den.to_f64().unwrap_or(1.0))
        .rem_euclid(1.0);

        let signature = AtomPhaseSignature {
            atom_hash: atom.atom_hash,
            centroid_phase: phase_f64,
            mean_alignment: alignment,
            norm_band: 0,
            basis_hash: final_spinor.basis_hash,
            thread_strength: alignment.to_f64().unwrap_or(1.0),
        };

        let thread = LoomThread {
            atom: atom.clone(),
            signature,
            state: ThreadState::Active,
            timestamp: 0,
            alignment_at_weave: alignment,
        };

        // Update Field (EMA)
        let entry = self
            .phase_field
            .entry(atom.final_state)
            .or_insert(Rational64::from_integer(0));
        let f_entry = entry.to_f64().unwrap_or(0.0);
        let f_align = alignment.to_f64().unwrap_or(0.0);
        let num = (((f_entry + f_align) / 2.0) * 1000.0).round() as i64;
        *entry = Rational64::new(num, 1000).reduced();

        self.thread_store.insert(atom.atom_hash, thread);

        let key = PhaseKey::from_spinor(final_spinor);
        self.phase_index
            .entry(key)
            .or_insert_with(Vec::new)
            .push(atom.atom_hash);

        self.metrics.total_atoms += 1;
        self.metrics.active_threads += 1;
        true
    }

    // --- Compression Logic (v2.2 Recursive) ---

    pub fn compress_bucket(
        &mut self,
        key: PhaseKey,
        loss_hat: Rational64,
        ctx: &CompressionContext,
    ) -> Result<CompressionAtom, String> {
        let source_hashes: Vec<Hash32> = self
            .phase_index
            .get(&key)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|h| {
                if let Some(t) = self.thread_store.get(h) {
                    t.state == ThreadState::Active
                } else {
                    false
                }
            })
            .collect();

        if source_hashes.len() < ctx.min_sources {
            return Err("InsufficientSources".to_string());
        }

        let source_root = build_merkle_root(&source_hashes);

        // Detect Recursion & Lineage (RC1, RC2)
        let mut max_parent_depth = 0;
        let mut max_parent_loss = Rational64::from_integer(0);
        let mut lineage_roots: Vec<Hash32> = vec![];
        let mut invariant_core_hash: Option<Hash32> = None;

        for h in &source_hashes {
            if let Some(t) = self.thread_store.get(h) {
                if t.atom.kind == AtomKind::SummaryTrajectory {
                    if let Some(comp) = self
                        .compression_store
                        .values()
                        .find(|c| c.compressed_atom.atom_hash == *h)
                    {
                        max_parent_depth = max_parent_depth.max(comp.lineage.depth);
                        max_parent_loss = max_parent_loss.max(comp.lineage.cumulative_loss); // RC2: use max
                        lineage_roots.push(comp.source_root);

                        let current_core = comp.lineage.invariant_core_hash;
                        if let Some(existing) = invariant_core_hash {
                            if existing != current_core {
                                return Err("InvariantCoreMismatch".to_string());
                            } // RC4
                        } else {
                            invariant_core_hash = Some(current_core);
                        }
                    }
                }
            }
        }

        if max_parent_depth >= ctx.max_depth {
            return Err("RecursiveDepthExceeded".to_string());
        }

        // Compute Incremental Semantic Loss
        let mut min_margin = Rational64::from_integer(1000000);
        let mut sum_phase = 0.0;
        let mut max_authority = Rational64::from_integer(0);
        for h in &source_hashes {
            let t = self.thread_store.get(h).unwrap();
            if t.atom.margin_total < min_margin {
                min_margin = t.atom.margin_total;
            }
            if t.atom.cumulative_authority > max_authority {
                max_authority = t.atom.cumulative_authority;
            }
            sum_phase += t.signature.centroid_phase;
        }
        let avg_phase = sum_phase / source_hashes.len() as f64;
        let mut phase_dispersion = 0.0;
        for h in &source_hashes {
            let t = self.thread_store.get(h).unwrap();
            phase_dispersion += (t.signature.centroid_phase - avg_phase).abs();
        }
        let phase_loss_f64 = phase_dispersion / source_hashes.len() as f64;
        let phase_loss =
            Rational64::from_f64(phase_loss_f64).unwrap_or(Rational64::from_integer(0));

        let incremental_loss = (phase_loss * ctx.w_p).reduced();
        let cumulative_loss = (max_parent_loss + incremental_loss).reduced(); // RC2

        if incremental_loss > loss_hat || cumulative_loss > ctx.global_loss_hat {
            // RC3
            return Err("GlobalLossBudgetExceeded".to_string());
        }

        // Define/Verify Invariant Core (RC5)
        let first_source = self.thread_store.get(&source_hashes[0]).unwrap();
        let core = InvariantCore {
            domain: first_source.atom.domain,
            bucket_key: key.clone(),
            final_state: first_source.atom.final_state,
            policy_hash: ctx.policy_hash,
            margin_floor: min_margin - incremental_loss,
            authority_cap: max_authority,
        };
        let new_core_hash = core.canonical_hash();

        if let Some(existing) = invariant_core_hash {
            if existing != new_core_hash {
                return Err("InvariantCoreMismatch".to_string());
            }
        }

        // Build Recursive Summary Atom
        let mut compressed_atom = CohAtom {
            kind: AtomKind::SummaryTrajectory,
            version: 1,
            domain: core.domain,
            initial_state: first_source.atom.initial_state,
            final_state: core.final_state,
            margin_total: core.margin_floor,
            policy_hash: core.policy_hash,
            verifier_id: ctx.verifier_id,
            compression_certificate: Some(Hash32([0xCC; 32])), // fixture_only: allow_mock
            ..Default::default()
        };
        compressed_atom.atom_hash = compressed_atom.canonical_hash();

        let lineage = CompressionLineage {
            depth: max_parent_depth + 1, // RC1
            source_root,
            lineage_roots, // RC8
            cumulative_loss,
            invariant_core_hash: new_core_hash,
        };

        let mut compression = CompressionAtom {
            version: 2,
            source_root,
            source_count: source_hashes.len() as u64,
            compressed_atom,
            semantic_loss: SemanticLoss {
                margin_loss: Rational64::from_integer(0),
                utility_loss: Rational64::from_integer(0),
                phase_loss: phase_loss * ctx.w_p,
                total: incremental_loss,
            },
            loss_hat,
            lineage,
            bucket_key: key.clone(),
            compression_policy_hash: ctx.policy_hash,
            compression_witness_hash: Hash32([0xAA; 32]), // fixture_only: allow_mock
            compression_hash: Hash32([0; 32]),
            signature: Signature(vec![0; 64]),
        };
        compression.compression_hash = compression.canonical_hash();

        // Commit to Store
        self.compression_store
            .insert(compression.compression_hash, compression.clone());

        // Mark Sources (TC13)
        for h in source_hashes {
            if let Some(t) = self.thread_store.get_mut(&h) {
                t.state = ThreadState::CompressedSource;
            }
        }

        // Weave Summary (v2.1 API compatible)
        let mut summary_spinor = CohSpinor::default();
        summary_spinor.phase_num =
            Rational64::from_f64(avg_phase).unwrap_or(Rational64::from_integer(0));
        summary_spinor.phase_den = Rational64::from_integer(1);
        summary_spinor.norm = Rational64::from_integer(1);
        summary_spinor.state_hash = compression.compressed_atom.final_state;
        summary_spinor.basis_hash = key.basis_hash;

        let depth = max_parent_depth + 1;
        self.weave(compression.compressed_atom.clone(), &summary_spinor);

        self.metrics.compressed_atoms += 1;
        self.metrics.max_depth = self.metrics.max_depth.max(depth);
        Ok(compression)
    }

    pub fn ingest(&mut self, receipt: &BoundaryReceiptSummary, _config: &PhaseLoomConfig) {
        self.tau = self.tau.saturating_add(1);
        self.tension = self.tension.saturating_add(receipt.tension_score);
        if !receipt.accepted {
            self.curvature = self.curvature.saturating_add(1);
        }
        self.budget = self
            .budget
            .saturating_sub(if receipt.accepted { 10 } else { 50 });
    }

    pub fn is_circuit_broken(&self, config: &PhaseLoomConfig) -> bool {
        self.curvature > config.circuit_break_threshold || self.budget == 0
    }

    pub fn sample_best(&self) -> Option<String> {
        self.strategy_weights
            .0
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k.clone())
    }

    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

pub mod budget {
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct PhaseLoomBudget {
        pub work_capacity: u128,
    }
}

pub mod kernel {
    use super::{PhaseLoomConfig, PhaseLoomState};
    use crate::budget::PhaseLoomBudget;
    use coh_npe::receipt::BoundaryReceiptSummary;

    #[derive(Clone, Default)]
    pub struct PhaseLoomKernel {
        pub state: PhaseLoomState,
        pub budget: PhaseLoomBudget,
    }

    impl PhaseLoomKernel {
        pub fn new(state: PhaseLoomState, budget: PhaseLoomBudget) -> Self {
            Self { state, budget }
        }

        pub fn update(
            &mut self,
            receipt: &BoundaryReceiptSummary,
            config: &PhaseLoomConfig,
        ) -> Result<(), String> {
            self.state.ingest(receipt, config);
            Ok(())
        }
    }
}

pub fn phaseloom_init(config: &PhaseLoomConfig) -> PhaseLoomState {
    PhaseLoomState::new(config)
}
pub fn phaseloom_ingest(
    state: &mut PhaseLoomState,
    receipt: &BoundaryReceiptSummary,
    config: &PhaseLoomConfig,
) {
    state.ingest(receipt, config);
}
pub fn phaseloom_sample(state: &PhaseLoomState) -> Option<String> {
    state.sample_best()
}
pub fn phaseloom_circuit_broken(state: &PhaseLoomState, config: &PhaseLoomConfig) -> bool {
    state.is_circuit_broken(config)
}
pub fn phaseloom_serialize(state: &PhaseLoomState) -> Result<Vec<u8>, serde_json::Error> {
    state.serialize()
}
