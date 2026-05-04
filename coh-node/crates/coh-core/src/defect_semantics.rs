//! Defect Semantic Types
//!
//! Splits the overloaded "defect" concept into four distinct layers.
//! Per audit Patch 4:
//!
//! | Symbol | Meaning | Use |
//! |--------|---------|-----|
//! | δ_raw | observed/measured defect | evidence, not accounting |
//! | δ_hat | certified upper estimate | envelope certification |
//! | ρ (reserve) | spendable reserve | accounting law only! |
//! | Δ_max | maximum allowed | policy cap |
//!
//! Ladder: δ_raw ≤ δ_hat ≤ ρ ≤ Δ_max

use serde::{Deserialize, Serialize};

/// Defect semantic ladder with four tiers
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DefectSemantics {
    /// δ_raw: observed/measured raw defect from execution
    /// This is evidence, not used in accounting law
    pub raw_defect: u128,

    /// δ_hat: certified upper bound on hidden defect
    /// From semantic envelope, used in certification check
    pub delta_hat: u128,

    /// ρ (defect_reserve): spendable reserve for accounting
    /// This is what appears in: v_post + spend ≤ v_pre + reserve + authority
    pub defect_reserve: u128,

    /// Δ_max: maximum defect allowed by policy/profile
    /// Hard cap from policy configuration
    pub defect_cap: u128,
}

/// Normalize defect semantics from incoming fields
///
/// Supports migration: old `defect` field maps to `defect_reserve`
impl DefectSemantics {
    /// Create from fields, supporting legacy migration
    pub fn from_fields(
        legacy_defect: Option<u128>,
        raw_defect: Option<u128>,
        delta_hat: Option<u128>,
        defect_reserve: Option<u128>,
        defect_cap: Option<u128>,
    ) -> Self {
        // Priority: explicit > legacy alias > computed defaults
        let reserve = defect_reserve.or(legacy_defect).unwrap_or(0);

        let hat = delta_hat.unwrap_or(reserve);
        let raw = raw_defect.unwrap_or(0);
        let cap = defect_cap.unwrap_or(reserve);

        Self {
            raw_defect: raw,
            delta_hat: hat,
            defect_reserve: reserve,
            defect_cap: cap,
        }
    }

    /// Check semantic envelope: δ_raw ≤ δ_hat ≤ ρ ≤ Δ_max
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.raw_defect > self.delta_hat {
            return Err("raw_defect exceeds delta_hat");
        }
        if self.delta_hat > self.defect_reserve {
            return Err("delta_hat exceeds reserve");
        }
        if self.defect_reserve > self.defect_cap {
            return Err("reserve exceeds cap");
        }
        Ok(())
    }

    /// Get the value that belongs in accounting law
    pub fn for_accounting(&self) -> u128 {
        self.defect_reserve
    }

    /// Get the value for envelope certification  
    pub fn for_envelope(&self) -> u128 {
        self.delta_hat
    }

    /// Get the raw evidence value
    pub fn for_evidence(&self) -> u128 {
        self.raw_defect
    }
}

/// Metrics wire with defect clarity
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DefectMetricsWire {
    #[serde(default)]
    pub spend: String,

    // Legacy: maps to defect_reserve during migration
    #[serde(default, alias = "defect")]
    pub defect: Option<String>,

    #[serde(default)]
    pub raw_defect: Option<String>,

    #[serde(default)]
    pub delta_hat: Option<String>,

    #[serde(default)]
    pub defect_reserve: Option<String>,

    #[serde(default)]
    pub defect_cap: Option<String>,

    #[serde(default)]
    pub authority: String,
}

impl DefectMetricsWire {
    pub fn to_semantics(&self) -> DefectSemantics {
        fn parse_u128(s: &Option<String>) -> Option<u128> {
            s.as_ref().and_then(|v| v.parse().ok())
        }

        DefectSemantics::from_fields(
            parse_u128(&self.defect),
            parse_u128(&self.raw_defect),
            parse_u128(&self.delta_hat),
            parse_u128(&self.defect_reserve),
            parse_u128(&self.defect_cap),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_migration() {
        // Old format: defect = "100"
        let wire = DefectMetricsWire {
            defect: Some("100".to_string()),
            ..Default::default()
        };

        let sem = wire.to_semantics();
        assert_eq!(sem.defect_reserve, 100);
        assert_eq!(sem.raw_defect, 0);
    }

    #[test]
    fn test_explicit_fields() {
        let wire = DefectMetricsWire {
            defect: Some("100".to_string()), // legacy
            raw_defect: Some("50".to_string()),
            delta_hat: Some("80".to_string()),
            defect_reserve: Some("90".to_string()), // explicit override
            ..Default::default()
        };

        let sem = wire.to_semantics();
        assert_eq!(sem.raw_defect, 50);
        assert_eq!(sem.delta_hat, 80);
        assert_eq!(sem.defect_reserve, 90); // explicit takes priority
    }

    #[test]
    fn test_semantic_ladder() {
        let sem = DefectSemantics {
            raw_defect: 50,
            delta_hat: 80,
            defect_reserve: 90,
            defect_cap: 100,
        };

        assert!(sem.validate().is_ok());

        // Invalid: raw > hat
        let bad = DefectSemantics {
            raw_defect: 100,
            delta_hat: 80,
            defect_reserve: 90,
            defect_cap: 100,
        };
        assert!(bad.validate().is_err());
    }
}
