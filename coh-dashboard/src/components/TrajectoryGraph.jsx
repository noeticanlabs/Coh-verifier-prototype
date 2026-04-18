import React, { useMemo } from 'react';

const WITNESS_LABELS = ['C1', 'C2', 'C3', 'C4', 'C5', 'C6'];

const TrajectoryGraph = ({ candidates, selectedId, onSelect }) => {
  // SVG Layout Constants
  const width = 800;
  const height = 450;
  const padding = 60;
  const nodeRadius = 5;
  
  const maxDepth = Math.max(...candidates.map(c => c.depth), 1);
  const stepX = (width - padding * 2) / maxDepth;

  const trajectories = useMemo(() => {
    return candidates.map(tau => {
      const isSelected = tau.id === selectedId;
      const points = tau.receipts.map((r, i) => {
        const x = padding + i * stepX;
        // Deterministic spread
        const depthNodes = candidates.filter(c => c.depth === tau.depth);
        const siblingIndex = depthNodes.indexOf(tau);
        const totalSiblings = depthNodes.length;
        const spread = 80;
        const y = height / 2 + (siblingIndex - (totalSiblings - 1) / 2) * spread;
        return { x, y };
      });

      return {
        ...tau,
        points,
        isSelected
      };
    });
  }, [candidates, selectedId, stepX]);

  return (
    <div className="trajectory-viewport">
      <svg width="100%" height="100%" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="xMidYMid meet">
        <defs>
          <filter id="glow-brand">
            <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
            <feMerge>
              <feMergeNode in="coloredBlur"/>
              <feMergeNode in="SourceGraphic"/>
            </feMerge>
          </filter>
        </defs>

        {trajectories.map(tau => {
          const { points, isSelectable, isSelected, firstFailureIndex } = tau;
          const truncateAt = !isSelectable && firstFailureIndex !== null ? firstFailureIndex + 1 : points.length;
          const visiblePoints = points.slice(0, truncateAt);

          return (
            <g key={tau.id} onClick={() => onSelect(tau.id)} style={{ cursor: 'pointer' }}>
              {/* Path Line */}
              <polyline
                points={visiblePoints.map(p => `${p.x},${p.y}`).join(' ')}
                fill="none"
                stroke={isSelected ? 'var(--brand-primary)' : isSelectable ? 'var(--text-muted)' : 'var(--brand-blocked)'}
                strokeWidth={isSelected ? 3 : 1.5}
                strokeDasharray={tau.warnCount > 0 ? '4,4' : '0'}
                filter={isSelected ? 'url(#glow-brand)' : 'none'}
                opacity={isSelected ? 1 : 0.3}
                style={{ transition: 'all 0.4s cubic-bezier(0.4, 0, 0.2, 1)' }}
              />

              {/* Path Status Indicator (Truncation) */}
              {!isSelectable && firstFailureIndex !== null && (
                <g transform={`translate(${points[firstFailureIndex].x}, ${points[firstFailureIndex].y})`}>
                  <circle r="10" fill="hsla(350, 70%, 50%, 0.1)" stroke="var(--brand-blocked)" strokeWidth="1" strokeDasharray="2,2" />
                  <line x1="-4" y1="-4" x2="4" y2="4" stroke="var(--brand-blocked)" strokeWidth="1.5" />
                  <line x1="4" y1="-4" x2="-4" y2="4" stroke="var(--brand-blocked)" strokeWidth="1.5" />
                </g>
              )}

              {/* Nodes */}
              {visiblePoints.map((p, i) => (
                <g key={`n-${i}`} transform={`translate(${p.x}, ${p.y})`}>
                  <circle 
                    r={isSelected ? nodeRadius + 1 : nodeRadius} 
                    fill={isSelected ? 'var(--brand-primary)' : 'var(--bg-surface-elevated)'} 
                    stroke={tau.witnesses?.[i]?.c4?.status === 'pass' ? 'var(--brand-primary)' : 'var(--brand-blocked)'}
                    strokeWidth="1.5"
                    style={{ transition: 'transform 0.2s' }}
                  />
                  
                  {/* Step ID Label on hover/select */}
                  {isSelected && (
                    <text 
                      y="-12" 
                      textAnchor="middle" 
                      fontSize="8" 
                      fill="var(--text-secondary)" 
                      className="monospace" 
                      style={{ pointerEvents: 'none' }}
                    >
                      STEP_{i}
                    </text>
                  )}
                </g>
              ))}
            </g>
          );
        })}
      </svg>
    </div>
  );
};

export default TrajectoryGraph;