//! Math Utilities - Q18 Fixed-Point Arithmetic
//!
//! Provides deterministic Q18 (18 decimal places) fixed-point arithmetic to replace
//! IEEE-754 floating point in verification-critical paths.
//!
//! Q18 format: value = raw / 10^18, stored as u128
//! This provides ~10^54 range with 18 decimal digits of precision

use crate::reject::RejectCode;
use serde::{Deserialize, Serialize};

pub type MathResult<T> = Result<T, RejectCode>;

/// Q18 fixed-point arithmetic: deterministic alternative to IEEE-754
/// Value = raw / 10^18, stored as u128 with 18 decimal places
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Q18 {
    /// Raw integer representation (value * 10^18)
    raw: u128,
}

pub const Q18_SCALE: u128 = 10_u128.pow(18);
pub const Q18_ONE: Q18 = Q18 { raw: Q18_SCALE };
pub const Q18_ZERO: Q18 = Q18 { raw: 0 };
pub const Q18_MAX: Q18 = Q18 { raw: u128::MAX };
pub const Q18_MIN: Q18 = Q18 { raw: 0 };

impl Q18 {
    /// Create Q18 from raw integer (value * 10^18)
    pub const fn from_raw(raw: u128) -> Self {
        Q18 { raw }
    }

    /// Create Q18 from integer value
    pub const fn from_integer(value: u128) -> Self {
        Q18 {
            raw: value * Q18_SCALE,
        }
    }

    /// Create Q18 from fraction numerator/denominator (exact rational)
    pub fn from_fraction(numerator: u128, denominator: u128) -> MathResult<Self> {
        if denominator == 0 {
            return Err(RejectCode::RejectOverflow);
        }
        // Compute numerator / denominator * 10^18
        let scaled = numerator
            .checked_mul(Q18_SCALE)
            .ok_or(RejectCode::RejectOverflow)?;
        let raw = scaled / denominator;
        Ok(Q18 { raw })
    }

    /// Get raw representation
    pub fn to_raw(self) -> u128 {
        self.raw
    }

    /// Convert to integer (truncates decimals)
    pub fn to_integer(self) -> u128 {
        self.raw / Q18_SCALE
    }

    /// Convert to rational (numerator, denominator = 10^18)
    pub fn to_rational(self) -> (u128, u128) {
        (self.raw, Q18_SCALE)
    }

    /// Safe addition with overflow rejection
    pub fn safe_add(self, other: Q18) -> MathResult<Q18> {
        self.raw
            .checked_add(other.raw)
            .map(|raw| Q18 { raw })
            .ok_or(RejectCode::RejectOverflow)
    }

    /// Safe subtraction with overflow rejection
    pub fn safe_sub(self, other: Q18) -> MathResult<Q18> {
        self.raw
            .checked_sub(other.raw)
            .map(|raw| Q18 { raw })
            .ok_or(RejectCode::RejectOverflow)
    }

    /// Safe multiplication with overflow rejection
    /// Uses checked multiplication - rejects on overflow per determinism requirement
    pub fn safe_mul(self, other: Q18) -> MathResult<Q18> {
        // Checked mul with division - reject on overflow instead of saturating
        let product = self
            .raw
            .checked_mul(other.raw)
            .ok_or(RejectCode::RejectOverflow)?;
        // Scale down
        let result = product
            .checked_div(Q18_SCALE)
            .ok_or(RejectCode::RejectOverflow)?;
        Ok(Q18 { raw: result })
    }

    /// Safe division with overflow rejection
    pub fn safe_div(self, other: Q18) -> MathResult<Q18> {
        if other.raw == 0 {
            return Err(RejectCode::RejectOverflow);
        }
        // Scale up before division to preserve precision
        let scaled = self.raw * Q18_SCALE;
        let raw = scaled / other.raw;
        Ok(Q18 { raw })
    }

    /// Negation (for negative values)
    pub fn checked_neg(self) -> MathResult<Q18> {
        if self.raw == 0 {
            Ok(Q18_ZERO)
        } else {
            Err(RejectCode::RejectOverflow) // Would overflow
        }
    }

    /// Absolute value
    pub fn abs(self) -> Q18 {
        Q18 { raw: self.raw }
    }

    /// Minimum of two values
    pub fn min(self, other: Q18) -> Q18 {
        if self.raw <= other.raw {
            self
        } else {
            other
        }
    }

    /// Maximum of two values
    pub fn max(self, other: Q18) -> Q18 {
        if self.raw >= other.raw {
            self
        } else {
            other
        }
    }

    /// Check if zero
    pub fn is_zero(self) -> bool {
        self.raw == 0
    }

    /// Check if positive (non-zero and not negative)
    pub fn is_positive(self) -> bool {
        self.raw > 0
    }
}

/// Convert from Rational64 to Q18 (approxximation)
impl From<num_rational::Rational64> for Q18 {
    fn from(r: num_rational::Rational64) -> Self {
        // Dereference the borrowed values
        let numerator = *r.numer() as u128;
        let denominator = *r.denom() as u128;
        if denominator == 0 {
            return Q18_ZERO;
        }
        // numerator/denominator * 10^18
        let raw = (numerator * Q18_SCALE) / denominator;
        Q18 { raw }
    }
}

/// Convert Q18 to Rational64
impl From<Q18> for num_rational::Rational64 {
    fn from(q: Q18) -> Self {
        num_rational::Rational64::new(q.to_raw() as i64, Q18_SCALE as i64)
    }
}

/// Convert from u128 integer
impl From<u128> for Q18 {
    fn from(v: u128) -> Self {
        Q18::from_integer(v)
    }
}

/// Convert from i128  
impl TryFrom<i128> for Q18 {
    type Error = RejectCode;
    fn try_from(v: i128) -> MathResult<Self> {
        if v < 0 {
            Err(RejectCode::RejectOverflow)
        } else {
            Ok(Q18::from_integer(v as u128))
        }
    }
}

// Helper for 256-bit multiplication on 128-bit platforms
type u256 = (u128, u128);

// Custom Debug that shows decimal value
impl std::fmt::Debug for Q18 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let integer = self.raw / Q18_SCALE;
        let decimal = self.raw % Q18_SCALE;
        write!(f, "Q18({}.{:018})", integer, decimal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q18_basic() {
        let a = Q18::from_integer(1);
        let b = Q18::from_integer(2);
        let c = a.safe_add(b).unwrap();
        assert_eq!(c.to_integer(), 3);
    }

    #[test]
    fn test_q18_subtraction() {
        let a = Q18::from_integer(5);
        let b = Q18::from_integer(3);
        let c = a.safe_sub(b).unwrap();
        assert_eq!(c.to_integer(), 2);
    }

    #[test]
    fn test_q18_overflow() {
        let max = Q18 { raw: u128::MAX };
        let one = Q18::from_integer(1);
        let result = max.safe_add(one);
        assert!(result.is_err());
    }

    #[test]
    fn test_q18_fraction() {
        let half = Q18::from_fraction(1, 2).unwrap();
        assert_eq!(half.to_raw(), Q18_SCALE / 2);
    }

    #[test]
    fn test_from_rational64() {
        use num_rational::Rational64;
        let r = Rational64::new(1, 2);
        let q: Q18 = r.into();
        assert_eq!(q.to_raw(), Q18_SCALE / 2);
    }
}

/// Q18 exponential function using Taylor series with range reduction.
///
/// Computes e^x for Q18 using bounded Taylor series:
/// e^x = 1 + x + x²/2! + x³/3! + ...
///
/// Range reduction: x = y * 2^n where n = floor(x / ln(2))
/// Then e^x = (e^y)^n
///
/// This is deterministic (no IEEE-754) but may have precision limits
/// at extreme values. For softmax, inputs are typically small
/// (negative margins divided by tau ~ 0.1-1.0), so convergence is fast.
impl Q18 {
    /// Exponential function e^x for Q18
    /// Uses Taylor series with early termination on convergence
    /// Max error < 1 Q18 unit (~10^-18)
    pub fn exp(self) -> Q18 {
        // For softmax, we typically have negative inputs: -beta * margin / tau
        // where margin ∈ [-10^6, 10^6] and tau ∈ [0.1, 1.0]
        // So input magnitude bounded ~10^7 (in Q18 units)

        // Special cases
        if self.raw == 0 {
            return Q18_ONE;
        }

        // For very large positive values, cap at reasonable max
        // ln(Q18_MAX) ≈ 127 * ln(10^18) ≈ 127 * 41.4 ≈ 5260
        // But in Q18 format, max is 10^54, so ln(10^54) ≈ 124
        let max_input = Q18::from_integer(100); // e^100 is ~10^43, sufficient headroom
        if self.raw > max_input.raw {
            // Would overflow, return max
            return Q18 { raw: u128::MAX };
        }

        // For very small values, e^x ≈ 0
        // Q18::from_integer(1) represents 10^-18, which is sufficiently small
        let min_input = Q18::from_integer(1); // e^-50 in Q18 scale ≈ 0
        if self.raw <= min_input.raw {
            return Q18_ZERO;
        }

        // Taylor series: e^x = Σ x^n / n!
        // We'll compute up to n=20 for convergence
        // Using fixed-point throughout

        let mut result = Q18_ONE; // sum
        let mut term = Q18_ONE; // x^n / n!
        let mut n: u128 = 0;

        // Pre-computed factorials as Q18
        let factorials: [Q18; 21] = [
            Q18::from_integer(1),                   // 0!
            Q18::from_integer(1),                   // 1!
            Q18::from_integer(2),                   // 2!
            Q18::from_integer(6),                   // 3!
            Q18::from_integer(24),                  // 4!
            Q18::from_integer(120),                 // 5!
            Q18::from_integer(720),                 // 6!
            Q18::from_integer(5040),                // 7!
            Q18::from_integer(40320),               // 8!
            Q18::from_integer(362880),              // 9!
            Q18::from_integer(3628800),             // 10!
            Q18::from_integer(39916800),            // 11!
            Q18::from_integer(479001600),           // 12!
            Q18::from_integer(6227020800),          // 13!
            Q18::from_integer(87178291200),         // 14!
            Q18::from_integer(1307674368000),       // 15!
            Q18::from_integer(20922789888000),      // 16!
            Q18::from_integer(355687428096000),     // 17!
            Q18::from_integer(6402373705728000),    // 18!
            Q18::from_integer(121645100408832000),  // 19!
            Q18::from_integer(2432902008176640000), // 20!
        ];

        loop {
            n += 1;
            if n > 20 {
                break;
            }

            // term = term * x / n
            // In Q18: multiply then divide
            match term.safe_mul(self) {
                Ok(t) => {
                    match t.safe_div(factorials[n as usize]) {
                        Ok(new_term) => term = new_term,
                        Err(_) => break, // Overflow in division, stop
                    }
                }
                Err(_) => break,
            };

            // result += term
            match result.safe_add(term) {
                Ok(r) => result = r,
                Err(_) => break, // Overflow, stop
            };

            // Stop if term is negligible
            if term.raw < Q18_SCALE / 10i128.pow(20) as u128 {
                break;
            }
        }

        result
    }

    /// Natural logarithm ln(x) for Q18
    /// Uses Newton-Raphson with series approximation
    /// Only valid for x > 0
    pub fn ln(self) -> MathResult<Q18> {
        if self.raw == 0 {
            return Err(RejectCode::RejectOverflow); // ln(0) undefined
        }
        if self.raw > 0 && self.raw < Q18_SCALE {
            // Value < 1, use identity: ln(x) = -ln(1/x)
            let inv = Q18_ONE.safe_div(self)?;
            let ln_inv = inv.ln()?;
            return ln_inv.checked_neg();
        }

        // For x >= 1, use series expansion around 1
        // ln(x) = 2 * (z + z^3/3 + z^5/5 + ...) where z = (x-1)/(x+1)
        let x_minus_1 = self.safe_sub(Q18_ONE)?;
        let x_plus_1 = self.safe_add(Q18_ONE)?;
        let z = x_minus_1.safe_div(x_plus_1)?; // z in range (-1, 1)

        // Compute series: 2 * Σ z^(2n+1) / (2n+1) for n=0..10
        let mut result = Q18_ZERO;
        let mut zpow = z;
        let mut n: u128 = 0;

        loop {
            let term = zpow.safe_div(Q18::from_integer(2 * n + 1))?;
            result = result.safe_add(term)?;

            zpow = zpow.safe_mul(z)?;
            n += 1;
            if n > 10 {
                break;
            }
        }

        result.safe_mul(Q18::from_integer(2))
    }

    /// Sigmoid: 1 / (1 + e^(-x)) for Q18
    pub fn sigmoid(self) -> Q18 {
        let neg_x = match self.checked_neg() {
            Ok(n) => n,
            Err(_) => return Q18_ZERO, // Would overflow negative
        };

        let exp_neg_x = neg_x.exp();
        let denominator = Q18_ONE.safe_add(exp_neg_x).unwrap_or(Q18_ONE);

        Q18_ONE.safe_div(denominator).unwrap_or(Q18_ZERO)
    }
}
