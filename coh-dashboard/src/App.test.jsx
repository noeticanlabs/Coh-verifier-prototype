import { describe, it, expect } from 'vitest';
import * as fs from 'node:fs/promises';
import * as path from 'node:path';

// Helper to read demo files directly from public folder
async function readDemoFile(filename) {
    const filePath = path.join(process.cwd(), 'public', filename);
    const content = await fs.readFile(filePath, 'utf-8');
    return content;
}

// Simple JSON Lines parser (same as cohData.js)
function parseJsonLines(text) {
    return text
        .split('\n')
        .filter((line) => line.trim())
        .map((line) => JSON.parse(line));
}

describe('App with real demo data', () => {
    // Test 1: Parse the valid chain data
    it('parses valid_chain.jsonl correctly', async () => {
        const text = await readDemoFile('demo/valid_chain.jsonl');
        const lines = parseJsonLines(text);

        // Expect at least 2 receipts in the chain
        expect(lines.length).toBeGreaterThanOrEqual(2);
        expect(lines[0].schema_id).toBe('coh.receipt.micro.v1');
        expect(lines[0].object_id).toBeDefined();
    });

    // Test 2: Parse slab data and verify structure  
    it('parses slab valid data correctly', async () => {
        const text = await readDemoFile('demo/ai_workflow_slab_valid.json');
        const slab = JSON.parse(text);

        expect(slab.schema_id).toBe('coh.receipt.slab.v1');
        expect(slab.object_id).toBeDefined();
        expect(slab.summary).toBeDefined();
        expect(typeof slab.summary.total_spend).toBe('string');
    });

    // Test 3: Verify chain data is properly formatted  
    it('valid chain has correct receipt structure', async () => {
        const text = await readDemoFile('demo/valid_chain.jsonl');
        const receipts = parseJsonLines(text);

        for (const receipt of receipts) {
            expect(receipt.schema_id).toBe('coh.receipt.micro.v1');
            expect(receipt.version).toBe('1.0.0');
            expect(receipt.object_id).toBeDefined();
            expect(receipt.step_index).toBeDefined();
            expect(receipt.state_hash_prev).toBeDefined();
            expect(receipt.state_hash_next).toBeDefined();
            expect(receipt.chain_digest_prev).toBeDefined();
            expect(receipt.chain_digest_next).toBeDefined();
            expect(receipt.metrics).toBeDefined();
            expect(receipt.metrics.v_pre).toBeDefined();
            expect(receipt.metrics.v_post).toBeDefined();
        }
    });

    // Test 4: Verify slab summary has valid metrics
    it('slab summary has valid numeric metrics', async () => {
        const slabText = await readDemoFile('demo/ai_workflow_slab_valid.json');
        const slab = JSON.parse(slabText);

        // Verify each metric is a valid numeric string
        expect(parseInt(slab.summary.total_spend, 10)).toBeGreaterThanOrEqual(0);
        expect(parseInt(slab.summary.total_defect, 10)).toBeGreaterThanOrEqual(0);
        expect(parseInt(slab.summary.v_pre_first, 10)).toBeGreaterThan(0);
        expect(parseInt(slab.summary.v_post_last, 10)).toBeGreaterThan(0);
    });

    // Test 5: Verify reject data structure
    it('reject_policy_violation has valid rejection structure', async () => {
        const text = await readDemoFile('demo/reject_policy_violation.jsonl');
        const receipts = parseJsonLines(text);

        // Should have receipts with metrics
        expect(receipts.length).toBeGreaterThanOrEqual(1);
        expect(receipts[0].metrics).toBeDefined();
    });

    // Test 6: All demo files are accessible
    it('loads all demo scenarios from public folder', async () => {
        const scenarios = [
            'demo/valid_chain.jsonl',
            'demo/ai_workflow_slab_valid.json',
            'demo/reject_policy_violation.jsonl',
            'demo/reject_state_link.jsonl',
            'demo/reject_chain_digest.jsonl'
        ];

        for (const scenario of scenarios) {
            const content = await readDemoFile(scenario);
            expect(content.length, `Failed to load ${scenario}`).toBeGreaterThan(0);
        }
    });

    // Test 7: Verify cross-file consistency - slab summary reflects chain range
    it('slab range matches chain step indices', async () => {
        const chainText = await readDemoFile('demo/valid_chain.jsonl');
        const receipts = parseJsonLines(chainText);

        const slabText = await readDemoFile('demo/ai_workflow_slab_valid.json');
        const slab = JSON.parse(slabText);

        // Slab range_start should match first receipt step_index
        expect(slab.range_start).toBe(receipts[0].step_index);
        // Slab range_end should be >= last receipt step_index  
        expect(slab.range_end).toBeGreaterThanOrEqual(receipts[receipts.length - 1].step_index);
        // Micro count should be >= number of receipts
        expect(slab.micro_count).toBeGreaterThanOrEqual(receipts.length);
    });
});
