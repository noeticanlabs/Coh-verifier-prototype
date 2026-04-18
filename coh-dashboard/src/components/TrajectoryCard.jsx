import React from 'react';
import TrajectoryGraph from './TrajectoryGraph';
import { Search, Compass, List, FileCode, Landmark } from 'lucide-react';

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

export const TechnicalTabs = ({ chainSteps, selectedStepIndex, candidates, selectedId }) => {
  const [activeTab, setActiveTab] = React.useState('trace');
  const step = chainSteps[selectedStepIndex] || {};

  // Find selected trajectory for evidence panel
  const selectedTrajectory = candidates?.find(t => t.id === selectedId);

  const TabButton = ({ id, icon: Icon, label }) => (
    <button
      onClick={() => setActiveTab(id)}
      style={{
        padding: '0.5rem 1rem',
        background: 'transparent',
        border: 'none',
        borderBottom: activeTab === id ? '2px solid var(--brand-primary)' : '2px solid transparent',
        color: activeTab === id ? 'var(--text-primary)' : 'var(--text-muted)',
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        cursor: 'pointer',
        fontSize: '0.75rem',
        fontWeight: activeTab === id ? 700 : 400,
        transition: 'var(--transition)'
      }}
    >
      <Icon size={14} />
      {label}
    </button>
  );

  return (
    <div className="card" style={{ marginTop: '1.5rem', minHeight: '300px' }}>
      <div style={{ display: 'flex', borderBottom: '1px solid var(--border-muted)', background: 'var(--bg-surface-elevated)' }}>
        <TabButton id="trace" icon={List} label="Audit Trace" />
        <TabButton id="evidence" icon={Compass} label="Evidence" />
        <TabButton id="payload" icon={FileCode} label="Raw Payload" />
        <TabButton id="formal" icon={Landmark} label="Formal Warrant" />
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
                  <tr key={i} style={{ borderTop: '1px solid var(--border-muted)', background: selectedStepIndex === i ? 'var(--bg-base)' : 'transparent' }}>
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
            <div style={{ marginBottom: '1rem', fontSize: '0.85rem', fontWeight: 600 }}>
              Trajectory Evidence — {selectedTrajectory.id}
            </div>

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
