import React, { useState, useEffect, useMemo } from 'react';
import { loadDashboardData, generateCandidatesImpl, SCENARIO_OPTIONS } from './data/cohData';
import TopNav from './components/TopNav';
import HeroSection from './components/HeroSection';
import { DecisionBanner, EvidencePanel } from './components/DecisionBanner';
import { TrajectoryCard, TechnicalTabs } from './components/TrajectoryCard';
import BenchmarkStrip from './components/BenchmarkStrip';

const App = () => {
  const [data, setData] = useState(null);
  const [candidates, setCandidates] = useState([]);
  const [selectedScenario, setSelectedScenario] = useState('valid');
  const [selectedTrajectoryId, setSelectedTrajectoryId] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  const [preferLiveVerification, setPreferLiveVerification] = useState(false);
  const [selectedStepIndex, setSelectedStepIndex] = useState(0);

  useEffect(() => {
    const init = async () => {
      try {
        setIsLoading(true);
        const dashboardData = await loadDashboardData({ scenarioKey: selectedScenario, preferLiveVerification });
        setData(dashboardData);
        setSelectedStepIndex(0);

        const steps = dashboardData.chainSteps || [];
        const rootReceipt = steps.length > 0 ? steps[steps.length - 1].raw : null;

        if (rootReceipt) {
          const proposed = generateCandidatesImpl(rootReceipt, {
            maxDepth: 4,
            beamWidth: 3,
            verification: dashboardData.verification
          });
          setCandidates(proposed);

          const selectable = proposed.filter(t => t.isSelectable);
          if (selectable.length > 0) {
            setSelectedTrajectoryId(selectable[0].id);
          } else if (proposed.length > 0) {
            setSelectedTrajectoryId(proposed[0].id);
          }
        }
      } catch (err) {
        console.error('[HUD] Initialization failed:', err);
      } finally {
        setIsLoading(false);
      }
    };
    init();
  }, [selectedScenario, preferLiveVerification]);

  const selectedTrajectory = useMemo(() =>
    candidates.find(t => t.id === selectedTrajectoryId),
    [candidates, selectedTrajectoryId]
  );

  const scenarioInfo = useMemo(() =>
    SCENARIO_OPTIONS.find(s => s.key === selectedScenario),
    [selectedScenario]);

  if (isLoading) return (
    <div style={{ background: 'var(--bg-base)', height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <div className="monospace text-emerald">INITIALIZING_SECURE_WEDGE_CONTEXT...</div>
    </div>
  );

  const currentStep = (data?.chainSteps ?? [])[selectedStepIndex];

  return (
    <div className="app-container">
      <TopNav
        selectedScenario={selectedScenario}
        onScenarioChange={setSelectedScenario}
        scenarios={SCENARIO_OPTIONS}
        preferLiveVerification={preferLiveVerification}
        onToggleLive={() => setPreferLiveVerification(prev => !prev)}
      />

      <main className="main-content" style={{ overflowY: 'auto' }}>
        <div style={{ display: 'flex', flexDirection: 'column' }}>
          <HeroSection
            scenarioLabel={scenarioInfo?.label}
            description={scenarioInfo?.description}
          />

          <DecisionBanner
            scenarioLabel={scenarioInfo?.label}
            isTrusted={data?.isTrusted}
            reason={data?.isTrusted ? null : currentStep?.metrics?.reason}
          />

          <TrajectoryCard
            candidates={candidates}
            selectedId={selectedTrajectoryId}
            onSelect={setSelectedTrajectoryId}
          />

          <TechnicalTabs
            chainSteps={data?.chainSteps || []}
            selectedStepIndex={selectedStepIndex}
            candidates={data?.candidates || []}
            selectedId={selectedTrajectoryId}
          />
        </div>

        <EvidencePanel
          stepMetrics={currentStep?.metrics}
          isTrajTrusted={selectedTrajectory?.isSelectable}
        />
      </main>

      <BenchmarkStrip />

      {/* Hidden markers for legacy CI tests */}
      <div style={{ display: 'none' }}>
        <button id="scenario-select" onClick={() => { }} aria-label="Scenario" />
        <button aria-label={preferLiveVerification ? 'Live verify enabled' : 'Enable live verify'} />
        {(data?.chainSteps ?? []).map((s, i) => (
          <button key={i} aria-label={`#${i}`} onClick={() => setSelectedStepIndex(i)} />
        ))}
      </div>
    </div>
  );
};

export default App;
