import React from 'react';
import { ShieldCheck, Zap, Activity } from 'lucide-react';

const TopNav = ({ 
  selectedScenario, 
  onScenarioChange, 
  scenarios = [], 
  preferLiveVerification, 
  onToggleLive 
}) => {
  return (
    <nav className="top-nav">
      <div className="logo-container">
        <ShieldCheck className="text-emerald" size={24} />
        <span className="logo-text" style={{ letterSpacing: '0.1em' }}>NOETICAN_LABS // INTEGRITY_OPS</span>
        <div style={{ width: '1px', height: '20px', background: 'var(--border-muted)', margin: '0 0.5rem' }} />
        <span className="monospace" style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>COH_WEDGE_OS v2.4</span>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '2rem' }}>
        <div className="metric-group">
          <div className="metric-item">
            <span className="metric-label">System Mode</span>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <div className="live-indicator" />
              <span className="metric-value text-emerald">DETERMINISTIC_ENFORCEMENT</span>
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', padding: '0.5rem', background: 'var(--bg-base)', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-muted)' }}>
          <label htmlFor="scenario-select" className="metric-label" style={{ margin: 0 }}>Active Workflow</label>
          <select
            id="scenario-select"
            value={selectedScenario}
            onChange={(e) => onScenarioChange(e.target.value)}
            style={{ 
              background: 'transparent', 
              color: 'var(--text-primary)', 
              border: 'none', 
              fontFamily: 'var(--font-mono)',
              fontSize: '0.8rem',
              outline: 'none',
              cursor: 'pointer'
            }}
          >
            {scenarios.map(opt => (
              <option key={opt.key} value={opt.key}>{opt.label}</option>
            ))}
          </select>
        </div>

        <button
          onClick={onToggleLive}
          className={`monospace`}
          style={{
            background: preferLiveVerification ? 'var(--brand-primary)' : 'var(--bg-surface-elevated)',
            color: preferLiveVerification ? 'black' : 'var(--text-primary)',
            border: '1px solid var(--border-bright)',
            padding: '6px 12px',
            borderRadius: 'var(--radius-sm)',
            fontSize: '0.7rem',
            fontWeight: 700,
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            cursor: 'pointer',
            transition: 'var(--transition)'
          }}
        >
          <Zap size={14} fill={preferLiveVerification ? 'black' : 'none'} />
          {preferLiveVerification ? 'LIVE_VERIFY: ON' : 'ENABLE_LIVE_VERIFY'}
        </button>
      </div>
    </nav>
  );
};

export default TopNav;
