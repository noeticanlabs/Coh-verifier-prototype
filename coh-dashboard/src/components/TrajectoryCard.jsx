import React from 'react';
import TrajectoryGraph from './TrajectoryGraph';
import { List, Compass, FileCode, Landmark, Search } from 'lucide-react';

// TabButton component - defined outside for stability
const TabButton = ({ icon, label, isActive, onClick }) => {
  const Icon = icon;
  return (
    <button
      onClick={onClick}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.375rem',
        padding: '0.5rem 0.875rem',
        background: isActive ? 'var(--bg-surface-elevated)' : 'transparent',
        border: 'none',
        borderBottom: isActive ? '2px solid var(--brand-primary)' : '2px solid transparent',
        color: isActive ? 'var(--text-primary)' : 'var(--text-secondary)',
        fontSize: '0.8125rem',
        fontWeight: 500,
        cursor: 'pointer',
        transition: 'all 0.15s ease'
      }}
    >
      <Icon size={14} />
      <span>{label}</span>
    </button>
  );
};

export const TrajectoryCard = ({ candidates, selectedId, onSelect }) => {
  return (
    <div className="card" style={{ flex: 1 }}>
      <div className="card-header">
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
          <Compass size={18} className="text-emerald" />
          <span className="card-title">Live Execution Path Analysis</span>
        </div>
        <div className="status-pill success" style={{ fontSize: '0.6rem' }}>
          <Search size={10} />
          <span>Active Beam Search (w=3)</span>
        </div>
      </div>
      <div className="card-body" style={{ padding: 0, position: 'relative', overflow: 'hidden' }}>
        <TrajectoryGraph
          candidates={candidates}
          selectedId={selectedId}
          onSelect={onSelect}
        />

        {/* Legend Overlay */}
        <div style={{
          position: 'absolute',
          bottom: '1rem',
          left: '1rem',
          background: 'var(--bg-glass)',
          backdropFilter: 'blur(8px)',
          border: '1px solid var(--border-muted)',
          padding: '0.75rem',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.5rem'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '0.65rem' }}>
            <div style={{ width: '8px', height: '8px', background: 'var(--brand-primary)', borderRadius: '50%' }} />
            <span>Admissible Node</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '0.65rem' }}>
            <div style={{ width: '8px', height: '8px', background: 'var(--brand-blocked)', borderRadius: '50%' }} />
            <span>Policy Violation</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '0.65rem' }}>
            <div style={{ width: '12px', height: '2px', background: 'var(--brand-primary)' }} />
            <span>Verified Transition</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export const TechnicalTabs = ({ chainSteps, selectedStepIndex, onStepSelect, candidates, selectedId }) => {
  const [activeTab, setActiveTab] = React.useState('trace');
  const step = chainSteps[selectedStepIndex] || {};

  // Find selected trajectory for evidence panel
  const selectedTrajectory = candidates?.find(t => t.id === selectedId);

  return (
    <div className="card" style={{ marginTop: '1.5rem', minHeight: '300px' }}>
      <div style={{ display: 'flex', borderBottom: '1px solid var(--border-muted)', background: 'var(--bg-surface-elevated)' }}>
        <TabButton id="trace" icon={List} label="Audit Trace" isActive={activeTab === 'trace'} onClick={() => setActiveTab('trace')} />
        <TabButton id="evidence" icon={Compass} label="Evidence" isActive={activeTab === 'evidence'} onClick={() => setActiveTab('evidence')} />
        <TabButton id="payload" icon={FileCode} label="Raw Payload" isActive={activeTab === 'payload'} onClick={() => setActiveTab('payload')} />
        <TabButton id="formal" icon={Landmark} label="Formal Warrant" isActive={activeTab === 'formal'} onClick={() => setActiveTab('formal')} />
      </div>

      <div className="card-body">
        {activeTab === 'trace' && (
          <div className="monospace" style={{ fontSize: '0.8rem' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr style={{ textAlign: 'left', color: 'var(--text-muted)' }}>
                  <th style={{ paddingBottom: '0.5rem' }}>Idx</th>
                  <th style={{ paddingBottom: '0.5rem' }}>Object ID</th>
                  <th style={{ paddingBottom: '0.5rem' }}>Digest (Pre)</th>
                  <th style={{ paddingBottom: '0.5rem' }}>Status</th>
                </tr>
              </thead>
              <tbody>
                {chainSteps.map((s, i) => (
                  <tr
                    key={i}
                    onClick={() => onStepSelect?.(i)}
                    style={{
                      cursor: 'pointer',
                      borderTop: '1px solid var(--border-muted)',
                      background: selectedStepIndex === i ? 'var(--bg-base)' : 'transparent',
                      transition: 'background 0.2s ease'
                    }}
                  >
                    <td style={{ padding: '0.5rem 0' }}>#{i}</td>
                    <td style={{ padding: '0.5rem 0' }}>{s.objectId?.substring(0, 12)}...</td>
                    <td style={{ padding: '0.5rem 0', color: 'var(--text-muted)' }}>{s.stateHashPrev?.substring(0, 8)}...</td>
                    <td style={{ padding: '0.5rem 0' }}>
                      <span className={s.status === 'TRUSTED' ? 'text-emerald' : 'text-ruby'}>
                        {s.status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {activeTab === 'evidence' && selectedTrajectory && (
          <div style={{ padding: '1rem' }}>
            <div style={{ marginBottom: '1rem', fontSize: '0.85rem', fontWeight: 600, display: 'flex', justifyContent: 'space-between' }}>
              <span>Trajectory Evidence — {selectedId}</span>
              <span className={selectedTrajectory.isSelectable ? 'text-emerald' : 'text-ruby'}>
                {selectedTrajectory.pathStatus}
              </span>
            </div>

            {/* Lexicographic Evaluation Summary */}
            {selectedTrajectory.evaluation && (
                <div style={{ 
                    display: 'grid', 
                    gridTemplateColumns: 'repeat(3, 1fr)', 
                    gap: '1rem', 
                    marginBottom: '1.5rem',
                    padding: '0.75rem',
                    background: 'var(--bg-base)',
                    borderRadius: 'var(--radius-sm)',
                    border: '1px solid var(--border-muted)'
                }}>
                    <div style={{ textAlign: 'center' }}>
                        <div style={{ fontSize: '0.6rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Safety Bottleneck</div>
                        <div style={{ fontSize: '1.1rem', fontWeight: 700, color: selectedTrajectory.evaluation.safetyBottleneck > 0.8 ? 'var(--brand-primary)' : 'var(--brand-blocked)' }}>
                            {(selectedTrajectory.evaluation.safetyBottleneck * 100).toFixed(0)}%
                        </div>
                    </div>
                    <div style={{ textAlign: 'center' }}>
                        <div style={{ fontSize: '0.6rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Alignment Index</div>
                        <div style={{ fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-primary)' }}>
                            {(selectedTrajectory.evaluation.alignment * 100).toFixed(0)}%
                        </div>
                    </div>
                    <div style={{ textAlign: 'center' }}>
                        <div style={{ fontSize: '0.6rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Normalized Cost</div>
                        <div style={{ fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-secondary)' }}>
                            {(selectedTrajectory.evaluation.normalizedCost * 100).toFixed(0)}%
                        </div>
                    </div>
                </div>
            )}

            {/* Step-by-step constraint breakdown */}
            {selectedTrajectory.witnesses && selectedTrajectory.witnesses.map((witness, wi) => (
              <div key={wi} style={{
                marginBottom: '1rem',
                padding: '0.75rem',
                background: 'var(--bg-base)',
                borderRadius: 'var(--radius-sm)',
                borderLeft: witness?.C1?.status === 'fail' ? '3px solid var(--brand-blocked)' : '3px solid var(--brand-primary)'
              }}>
                <div style={{ fontSize: '0.75rem', fontWeight: 600, marginBottom: '0.5rem' }}>
                  Step {wi}: {selectedTrajectory.receipts?.[wi]?.objectId || `step_${wi}`}
                </div>

                {/* Constraint status grid */}
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '0.5rem', fontSize: '0.7rem' }}>
                  {['C1', 'C2', 'C3', 'C4', 'C5', 'C6'].map(c => {
                    const status = witness?.[c]?.status || 'unknown';
                    const isPass = status === 'pass';
                    const isFail = status === 'fail';
                    return (
                      <div key={c} style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '0.25rem',
                        color: isFail ? 'var(--brand-blocked)' : isPass ? 'var(--brand-primary)' : 'var(--text-muted)'
                      }}>
                        <span style={{
                          width: '6px',
                          height: '6px',
                          borderRadius: '50%',
                          background: isFail ? 'var(--brand-blocked)' : isPass ? 'var(--brand-primary)' : 'var(--text-muted)'
                        }} />
                        <span>{c}: {status}</span>
                      </div>
                    );
                  })}
                </div>

                {/* Receipt link */}
                {selectedTrajectory.receipts?.[wi] && (
                  <div style={{ marginTop: '0.5rem', fontSize: '0.65rem', color: 'var(--text-muted)' }}>
                    <span style={{ fontWeight: 600 }}>Receipt:</span>
                    <code style={{ marginLeft: '0.25rem' }}>
                      {selectedTrajectory.receipts[wi].schemaId}
                    </code>
                    <span style={{ marginLeft: '0.5rem' }}>Step: {selectedTrajectory.receipts[wi].stepIndex}</span>
                  </div>
                )}
              </div>
            ))}

            {!selectedTrajectory.witnesses && (
              <div style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
                No witness data available for selected trajectory.
              </div>
            )}
          </div>
        )}

        {activeTab === 'payload' && (
          <pre style={{
            fontSize: '0.75rem',
            color: 'var(--text-secondary)',
            background: 'var(--bg-base)',
            padding: '1rem',
            borderRadius: 'var(--radius-sm)',
            overflow: 'auto',
            maxHeight: '200px'
          }}>
            {JSON.stringify(step.raw || step, null, 2)}
          </pre>
        )}

        {activeTab === 'formal' && (
          <div style={{ fontSize: '0.85rem', lineHeight: 1.6 }}>
            <div className="metric-label" style={{ marginBottom: '1rem' }}>T-Stack Slab Grounding Proof</div>
            <p>
              This receipt forms a non-branching slab satisfying the <strong>Admissibility Persistence Theorem</strong>.
              The state transition is captured as an endmorphism in the Category of Coh-Governed Systems.
            </p>
            <div style={{ marginTop: '1rem', padding: '1rem', borderLeft: '3px solid var(--brand-primary)', background: 'hsla(150, 80%, 50%, 0.05)' }}>
              <em>&quot;For any object $X$ with potential $V(X)$, the transition $X \to Y$ is verified if $V(Y) \leq V(X) + \xi$ where $\xi$ is the measured authority.&quot;</em>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
