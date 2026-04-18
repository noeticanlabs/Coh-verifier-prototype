import React from 'react';
import { Box, CheckCircle, ShieldAlert, Cpu } from 'lucide-react';

const HeroSection = ({ scenarioLabel, description }) => {
  return (
    <div style={{ marginBottom: '2rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div>
          <h1 style={{ fontSize: '2.25rem', fontWeight: 800, marginBottom: '0.5rem', letterSpacing: '-0.02em' }}>
            Deterministic Execution Verification
          </h1>
          <p style={{ color: 'var(--text-secondary)', maxWidth: '600px', fontSize: '0.95rem', lineHeight: 1.6 }}>
            The Coh Safety Wedge enforces lawfulness across AI-driven and automated workflows. 
            By verifying each step against cryptographic invariants, we prevent inadmissible 
            actions from reaching operational systems.
          </p>
        </div>

        <div className="card" style={{ padding: '1rem', minWidth: '240px' }}>
          <div className="metric-item" style={{ marginBottom: '1rem' }}>
            <span className="metric-label">Integrity Status</span>
            <div className="status-pill success">
              <CheckCircle size={14} />
              <span>Hardened Kernel Active</span>
            </div>
          </div>
          <div className="metric-item">
            <span className="metric-label">Verification Mode</span>
            <span className="monospace" style={{ fontSize: '0.8rem' }}>JCS_ALIGNED_CANONICAL_JSON</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default HeroSection;
