/**
 * Verification Grade System
 * 
 * Defines strict verification hierarchy for dashboard display.
 * Only Rust/backend can produce CERTIFIED grade.
 * 
 * Per audit Priority 4: Dashboard verification grades
 */

export const VERIFICATION_GRADES = {
    /** No verification attempted - purely visual display */
    UNVERIFIED_DISPLAY: 'UNVERIFIED_DISPLAY',

    /** Structure is valid (can parse) */
    STRUCTURE_VALID: 'STRUCTURE_VALID',

    /** Local arithmetic validated */
    LOCAL_ARITHMETIC_VALID: 'LOCAL_ARITHMETIC_VALID',

    /** Digest matches */
    DIGEST_VALID: 'DIGEST_VALID',

    /** Signature verified */
    SIGNATURE_VALID: 'SIGNATURE_VALID',

    /** Chain continuity validated */
    CHAIN_VALID: 'CHAIN_VALID',

    /** Full Rust verifier certification */
    CERTIFIED: 'CERTIFIED',
};

/** Grade priority ordering (higher = more verified) */
export const GRADE_PRIORITY = {
    [VERIFICATION_GRADES.UNVERIFIED_DISPLAY]: 0,
    [VERIFICATION_GRADES.STRUCTURE_VALID]: 1,
    [VERIFICATION_GRADES.LOCAL_ARITHMETIC_VALID]: 2,
    [VERIFICATION_GRADES.DIGEST_VALID]: 3,
    [VERIFICATION_GRADES.SIGNATURE_VALID]: 4,
    [VERIFICATION_GRADES.CHAIN_VALID]: 5,
    [VERIFICATION_GRADES.CERTIFIED]: 6,
};

/**
 * Determine if grade implies cryptographic verification
 * @param {string} grade
 * @returns {boolean}
 */
export function isCryptographicallyVerified(grade) {
    return grade === VERIFICATION_GRADES.CERTIFIED;
}

/**
 * Determine if grade allows display
 * @param {string} grade
 * @returns {boolean}
 */
export function canDisplay(grade) {
    return GRADE_PRIORITY[grade] >= GRADE_PRIORITY[VERIFICATION_GRADES.STRUCTURE_VALID];
}

/**
 * Select appropriate grade string for display
 * Filters out false "certified" implications
 * 
 * @param {object} receiptData - Receipt from API
 * @returns {string} Safe grade for UI
 */
export function gradeForDisplay(receiptData) {
    // If no verification data, it's display only
    if (!receiptData || !receiptData.verified) {
        return VERIFICATION_GRADES.UNVERIFIED_DISPLAY;
    }

    // Map backend verification status to grade
    const verified = receiptData.verified;

    if (verified.certified === true) {
        return VERIFICATION_GRADES.CERTIFIED;
    }

    if (verified.chain_valid === true) {
        return VERIFICATION_GRADES.CHAIN_VALID;
    }

    if (verified.signature_valid === true) {
        return VERIFICATION_GRADES.SIGNATURE_VALID;
    }

    if (verified.digest_valid === true) {
        return VERIFICATION_GRADES.DIGEST_VALID;
    }

    if (verified.local_arithmetic_valid === true) {
        return VERIFICATION_GRADES.LOCAL_ARITHMETIC_VALID;
    }

    if (verified.structure_valid === true) {
        return VERIFICATION_GRADES.STRUCTURE_VALID;
    }

    return VERIFICATION_GRADES.UNVERIFIED_DISPLAY;
}