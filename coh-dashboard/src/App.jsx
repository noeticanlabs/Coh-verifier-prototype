import React, { useState, useEffect } from 'react';
import { 
  ShieldCheck, 
  ShieldAlert, 
  Activity, 
  ChevronRight, 
  Code, 
  Database,
  RefreshCw,
  Zap
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

const INITIAL_CHAIN = [
  { step: 0, status: 'TRUSTED', hash: '76114b520738a80e18048c2a37734c97b17d6f7e06f94393bbce7949bb87381c' },
  { step: 1, status: 'TRUSTED', hash: '4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09' },
  { step: 2, status: 'TRUSTED', hash: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855' }
];

export default function App() {
  const [chain, setChain] = useState(INITIAL_CHAIN);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [isSystemTrusted, setIsSystemTrusted] = useState(true);

  const triggerTamperDemo = () => {
    const newChain = chain.map((node, i) => 
      i >= 1 ? { ...node, status: 'TAMPERED' } : node
    );
    setChain(newChain);
    setIsSystemTrusted(false);
  };

  const resetChain = () => {
    setChain(INITIAL_CHAIN);
    setIsSystemTrusted(true);
    setSelectedIdx(0);
  };

  return (
    <div className="min-h-screen p-4 md:p-8 max-w-7xl mx-auto">
      {/* Header */}
      <header className="flex flex-col md:flex-row justify-between items-start md:items-center gap-6 mb-12">
        <div>
          <h1 className="text-3xl font-bold tracking-tight mb-2">
            Coh <span className="opacity-50">Integrity Dashboard</span>
          </h1>
          <div className="flex items-center gap-2 text-sm text-[#a0a0a0]">
            <Activity size={14} />
            <span>Operational Layer: Rust Kernel v0.1.0</span>
          </div>
        </div>
        
        <div className={`flex items-center gap-3 px-6 py-3 glass pulsate ${isSystemTrusted ? 'neon-green' : 'neon-red'}`}>
          {isSystemTrusted ? <ShieldCheck /> : <ShieldAlert />}
          <span className="font-mono font-bold uppercase tracking-widest text-lg">
            {isSystemTrusted ? 'TRUSTED' : 'TAMPERED'}
          </span>
        </div>
      </header>

      <main className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Main Flow Visualizer */}
        <div className="lg:col-span-2 space-y-8">
          <div className="glass p-8 relative overflow-hidden min-h-[300px]">
            <div className="absolute top-0 right-0 p-4">
              <RefreshCw 
                className="text-[#a0a0a0] cursor-pointer hover:rotate-90 transition-all" 
                size={18} 
                onClick={resetChain} 
              />
            </div>
            
            <h2 className="text-lg font-bold mb-8 uppercase tracking-widest opacity-50">Chain Continuity</h2>
            
            <div className="flex items-center gap-4 flex-wrap">
              {chain.map((node, i) => (
                <React.Fragment key={i}>
                  <motion.div 
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: i * 0.1 }}
                    onClick={() => setSelectedIdx(i)}
                    className={`h-16 w-16 glass flex items-center justify-center cursor-pointer transition-all hover:scale-110
                               ${selectedIdx === i ? 'border-[#00ffbd] border-2 shadow-lg' : ''}
                               ${node.status === 'TRUSTED' ? 'neon-green' : node.status === 'TAMPERED' ? 'neon-red' : ''}`}
                  >
                    <div className="text-sm font-bold font-mono">#{node.step}</div>
                  </motion.div>
                  {i < chain.length - 1 && (
                    <ChevronRight className={`opacity-20 ${chain[i+1].status === 'TAMPERED' ? 'text-[#ff4d4d] opacity-50' : ''}`} />
                  )}
                </React.Fragment>
              ))}
              
              <motion.button 
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                onClick={triggerTamperDemo}
                className="ml-auto px-4 py-2 bg-[#ff4d4d1a] border border-[#ff4d4d4d] rounded text-[#ff4d4d] text-xs font-bold uppercase tracking-tighter"
              >
                Simulate Tamper
              </motion.button>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="glass p-6">
              <div className="flex items-center gap-2 mb-4 opacity-50">
                <Database size={16} />
                <h3 className="text-xs font-bold uppercase tracking-widest">Receipt Digest</h3>
              </div>
              <p className="font-mono text-xs break-all leading-relaxed">
                {chain[selectedIdx]?.hash}
              </p>
            </div>
            <div className="glass p-6">
              <div className="flex items-center gap-2 mb-4 opacity-50">
                <Code size={16} />
                <h3 className="text-xs font-bold uppercase tracking-widest">Accounting Verification</h3>
              </div>
              <div className="space-y-2 font-mono text-xs">
                <div className="flex justify-between">
                  <span>v_pre</span>
                  <span className="text-[#00ffbd]">100.00</span>
                </div>
                <div className="flex justify-between">
                  <span>v_post</span>
                  <span>{chain[selectedIdx]?.status === 'TAMPERED' ? '200.00' : '88.00'}</span>
                </div>
                <div className="h-px bg-white/10 my-2" />
                <div className="flex justify-between font-bold">
                  <span>Status</span>
                  <span className={chain[selectedIdx]?.status === 'TAMPERED' ? 'text-[#ff4d4d]' : 'text-[#00ffbd]'}>
                    {chain[selectedIdx]?.status === 'TAMPERED' ? 'REJECTED' : 'ADMISSIBLE'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Sidebar Inspector */}
        <div className="space-y-8">
           <div className="glass p-6 h-full">
              <h3 className="text-lg font-bold mb-6 flex items-center gap-2">
                <Zap className="text-[#00ffbd]" size={20} />
                Audit Frame
              </h3>
              
              <div className="space-y-6">
                <div className="text-sm">
                  <span className="opacity-50 block mb-2 uppercase text-[10px] font-bold tracking-widest">Protocol Version</span>
                  <code className="bg-white/5 p-1 rounded">coh.receipt.micro.v1</code>
                </div>
                
                <div className="text-sm">
                  <span className="opacity-50 block mb-2 uppercase text-[10px] font-bold tracking-widest">Audit Policy</span>
                  <code className="bg-white/5 p-1 rounded text-[#a0a0a0]">deterministic_accounting_v1</code>
                </div>

                <div className="pt-6 border-t border-white/10">
                  <h4 className="text-xs font-bold mb-4 uppercase opacity-50">Internal Proof Vector</h4>
                  <pre className="text-[10px] overflow-hidden">
{JSON.stringify({
  request_id: "req_" + chain[selectedIdx]?.hash.slice(0, 8),
  decision: chain[selectedIdx]?.status === 'TAMPERED' ? 'REJECT' : 'ACCEPT',
  error_code: chain[selectedIdx]?.status === 'TAMPERED' ? 'E003' : null,
  provenance: "coh-sidecar:3030"
}, null, 2)}
                  </pre>
                </div>
              </div>
           </div>
        </div>
      </main>

      <footer className="mt-20 border-t border-white/5 pt-8 text-center text-xs text-[#a0a0a0] tracking-widest uppercase opacity-30">
        Coh © 2026 • Deterministic Verification Kernel • No-Bluff System
      </footer>
    </div>
  );
}
