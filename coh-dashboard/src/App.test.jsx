import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from '../App';

// Mock the data module to avoid external dependencies
vi.mock('../data/cohData', () => ({
    DEFAULT_SIDECAR_BASE_URL: 'http://localhost:3030',
    SCENARIO_OPTIONS: [
        { key: 'valid', label: 'Valid Chain' },
        { key: 'invalid_state_link', label: 'Invalid State Link' },
        { key: 'reject', label: 'Reject Cases' },
    ],
    loadDashboardData: vi.fn().mockResolvedValue({
        adapters: [
            {
                id: 'test-adapter',
                label: 'Test Adapter',
                scenarios: {
                    valid: {
                        steps: [
                            {
                                index: 0,
                                request_id: 'req-001',
                                state: 'State0',
                                digest: 'abc123',
                            },
                        ],
                    },
                },
            },
        ],
    }),
}));

describe('App', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders the main application', async () => {
        render(<App />);

        // Wait for loading to complete
        await waitFor(() => {
            expect(screen.queryByText('Loading...')).toBeNull();
        });

        // Check that the app title is rendered
        expect(screen.getByText(/Coherent/i)).toBeInTheDocument();
    });

    it('renders scenario options', async () => {
        render(<App />);

        await waitFor(() => {
            expect(screen.queryByText('Loading...')).toBeNull();
        });

        // Check scenario options exist
        expect(screen.getByText('Valid Chain')).toBeInTheDocument();
        expect(screen.getByText('Invalid State Link')).toBeInTheDocument();
    });

    it('switches scenarios when dropdown changes', async () => {
        const user = userEvent.setup();
        render(<App />);

        await waitFor(() => {
            expect(screen.queryByText('Loading...')).toBeNull();
        });

        // Click on the scenario dropdown
        const dropdown = screen.getByRole('combobox');
        await user.selectOptions(dropdown, 'invalid_state_link');

        // Verify the selection changed
        expect(screen.getByText('Invalid State Link')).toBeInTheDocument();
    });
});