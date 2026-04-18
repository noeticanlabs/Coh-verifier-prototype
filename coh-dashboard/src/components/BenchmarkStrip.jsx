import React from 'react';
import benchmarkData from '../data/benchmarkSnapshot.json';
import devData from '../data/benchmarkSnapshot.dev.json';
import { Cpu, Terminal, Clock, ChevronRight, Info } from 'lucide-react';

const BenchmarkStrip = () => {
  // Use CI data if populated, otherwise fallback to local dev reference
  const isCI = benchmarkData && benchmarkData.timestamp > 0;
  const data = isCI ? benchmarkData : devData;

  const formatOps = (val) => {
    if (!val || val === 0) return '---';
    if (val > 1000) return `${(val / 1000).toFixed(1)}k/sec`;
    return `${val.toFixed(0)}/sec`;
  };

  const formatDate = (ts) => {
    if (!ts || ts === 0) return 'PENDING';
    return new Date(ts * 1000).toLocaleString();
  };

  const stabilityScore = 100 - ((data.false_accept_rate || 0) + (data.false_reject_rate || 0)) * 100;

  return (
    <div 
      className="card" 
      style={{ 
        flexDirection: 'row', 
        alignItems: 'center', 
        gap: '3rem', 
        padding: '0.75rem 2rem', 
        background: 'var(--bg-surface-elevated)',
        borderRadius: 0,
        borderLeft: 'none',
        borderRight: 'none',
        borderBottom: 'none',
        borderTop: '1px solid var(--border-muted)'
      }}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.2rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
          <Terminal size={14} className={isCI ? "text-emerald" : "text-muted"} />
          <span className="monospace" style={{ fontSize: '0.65rem', color: isCI ? 'var(--text-primary)' : 'var(--text-muted)' }}>
            SNAPSHOT: {isCI ? 'CI_CANONICAL' : 'LOCAL_DEV_REFERENCE'}
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', opacity: 0.6 }}>
          <Info size={10} />
          <span style={{ fontSize: '0.6rem' }}>
            Source: {isCI ? 'Latest CI Benchmark' : 'Developer High-Perf Env'} | {formatDate(data.timestamp)}
          </span>
        </div>
      </div>

      <div className="metric-group" style={{ flex: 1, justifyContent: 'center' }}>
        <div className="metric-item">
          <span className="metric-label">Integrity Stability</span>
          <span className={`metric-value ${stabilityScore >= 100 ? 'text-emerald' : 'text-amber'}`}>
            {stabilityScore.toFixed(3)}%
          </span>
        </div>
        <div className="metric-item">
          <span className="metric-label">Peak Throughput</span>
          <span className="metric-value">{formatOps(data.concurrency_throughput_ops_sec)}</span>
        </div>
        <div className="metric-item">
          <span className="metric-label">Lat p99</span>
          <span className="metric-value monospace">&lt;{(data.p99_latency_us || 0).toFixed(0)}µs</span>
        </div>
        <div className="metric-item">
          <span className="metric-label">False Accept</span>
          <span className={`metric-value ${data.false_accept_rate > 0 ? 'text-blocked' : 'text-muted'}`}>
            {(data.false_accept_rate || 0).toFixed(4)}%
          </span>
        </div>
        <div className="metric-item">
          <span className="metric-label">False Reject</span>
          <span className={`metric-value ${data.false_reject_rate > 0 ? 'text-amber' : 'text-muted'}`}>
            {(data.false_reject_rate || 0).toFixed(4)}%
          </span>
        </div>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
        <div className="metric-item" style={{ alignItems: 'flex-end' }}>
          <span className="metric-label">Target Hardware</span>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <Cpu size={14} className="text-secondary" />
            <span className="monospace" style={{ fontSize: '0.7rem' }}>
              {data.hardware?.cpu_model?.split('(')[0] || '---'}
            </span>
          </div>
        </div>
        <ChevronRight size={16} className="text-muted" />
      </div>
    </div>
  );
};

export default BenchmarkStrip;
