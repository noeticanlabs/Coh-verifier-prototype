/**
 * Formula Redaction Middleware
 * 
 * Filters mathematical formulas, technical equations, and proprietary
 * trade secrets from dashboard API responses before rendering.
 * 
 * Per Constraint 6: Public-facing visualization must exclude sensitive data.
 */

import { useMemo } from 'react';

/**
 * Patterns that indicate sensitive mathematical content
 */
const SENSITIVE_PATTERNS = [
    /\$\$/,                    // LaTeX math delimiters
    /\$.*\$/,                  // Inline LaTeX
    /\\frac\{/,                // LaTeX fractions
    /\\sum\{/,                 // LaTeX summations
    /\\int\{/,                // LaTeX integrals
    /\\delta/,                 // Delta symbols
    /\\cdot|\\times|\\div/,   // Math operators
    /\\sqrt\{/,               // Square roots
    /\^/,                     // Superscripts in math context
    /_\d+/,                   // Subscripts
    /\bdelta_hat\b/,           // Delta-hat (envelope defect)
    /\bomega\b(?=.*\d)/,     // Omega constants
    /\blambda\b(?=.*\d)/,     // Lambda parameters
    /\btheta\b(?=.*\d)/,      // Theta parameters
    /\bpsi\b/,                // PSI (wave function)
    /\bphi\b/,                // Phi (phase)
    /\bCohBit\b.*=/,          // CohBit definitions
    /\bverifier\b.*=/,        // Verifier equations
    /\bmargin\b.*[+\-]/,      // Margin calculations
    /v_pre.*v_post/,         // Valuation equations
    /\bSpending\b.*Defect/,  // Spend/Defect equations
    /\bTrajectory\b.*probability/, // Trajectory probabilities
    /E\[.*\]/,               // Expectation values
    /\bsup\b_\w+\s*/,         // Supremum notation
    /\binf\b_\w+\s*/,        // Infimum notation
];

/**
 * Regex patterns for formulas in strings
 */
const FORMULA_REGEX = /([$][^$]+[$]|[\\][a-zA-Z]+\{[^}]+\}|[+\-*/=<>]\s*\d+\.?\d*\s*[+\-*/=<>])/g;

/**
 * Fields that contain sensitive mathematical content
 */
const SENSITIVE_FIELDS = [
    'formula',
    'equation',
    'derivation',
    'marginal_formula',
    'valuation_equation',
    'spend_defect_ratio',
    'delta_hat_computation',
    'coherence_bound',
    'trajectory_action',
    'path_integral',
    'partition_function',
    'hamiltonian',
    'lagrangian',
    'action_function',
];

/**
 * Redact sensitive mathematical content from a string
 * @param {string} text - Input text
 * @returns {string} Redacted text with formulas replaced
 */
export function redactFormulas(text) {
    if (!text || typeof text !== 'string') {
        return text;
    }

    // Check each sensitive pattern
    for (const pattern of SENSITIVE_PATTERNS) {
        if (pattern.test(text)) {
            // Replace with placeholder indicating redaction
            return '[REDACTED]';
        }
    }

    // Additional formula detection
    if (FORMULA_REGEX.test(text)) {
        FORMULA_REGEX.lastIndex = 0;
        return '[REDACTED]';
    }

    return text;
}

/**
 * Recursively redact sensitive fields from an object
 * @param {object} data - Input data object
 * @param {string} path - Current path for debugging
 * @returns {object} Redacted copy of object
 */
export function redactObject(data, path = '') {
    if (data === null || data === undefined) {
        return data;
    }

    if (typeof data === 'string') {
        return redactFormulas(data);
    }

    if (typeof data === 'number' || typeof data === 'boolean') {
        return data;
    }

    if (Array.isArray(data)) {
        return data.map((item, i) => redactObject(item, `${path}[${i}]`));
    }

    if (typeof data === 'object') {
        const redacted = {};

        for (const [key, value] of Object.entries(data)) {
            const newPath = path ? `${path}.${key}` : key;

            // Check if field name indicates sensitivity
            const isSensitive = SENSITIVE_FIELDS.some(field =>
                key.toLowerCase().includes(field.toLowerCase())
            );

            if (isSensitive) {
                redacted[key] = '[REDACTED]';
            } else {
                redacted[key] = redactObject(value, newPath);
            }
        }

        return redacted;
    }

    return data;
}

/**
 * Hook for use in React components
 * @param {object} data - API response data
 * @returns {object} Redacted data safe for display
 */
export function useRedactedData(data) {
    return useMemo(() => redactObject(data), [data]);
}

/**
 * Filter for API responses
 * Call this on any data before sending to frontend
 */
export function filterApiResponse(data) {
    return redactObject(data);
}