//! # Probes
//!
//! Advanced probes that use `negative-space-testing` to analyze document parsing.
//!
//! ## SpaceProbe
//!
//! Uses `SpaceMap` to track which document features have been "occupied" (tested) and which
//! remain in the "forbidden" zone (untested).
//!
//! ## ConservationProbe
//!
//! Uses `ConservationChecker` to ensure that formatting fidelity never decreases
//! across parsing runs — tables shouldn't lose columns, images shouldn't disappear.
//!
//! ## CrackleProbe
//!
//! Uses `CracklePhase` to accumulate parsed document metrics and detect patterns
//! in the aggregate — did any format degrade across a batch?

use negative_space_testing::{SpaceMap, ConservationChecker, CracklePhase, CathedralProbe};

/// Track which document features have been tested using SpaceMap.
pub struct SpaceProbe {
    map: SpaceMap<String, bool>,
    /// Track which features are still forbidden (untested).
    forbidden: Vec<String>,
    /// Track which features have been occupied (tested).
    tested: Vec<String>,
}

impl SpaceProbe {
    /// Create a new probe with all standard features marked as forbidden (untested).
    pub fn new() -> Self {
        let mut map = SpaceMap::new();
        let feature_names = [
            "text", "lists", "nested_lists", "tables", "images",
            "formulas", "code_blocks", "headers", "footnotes",
            "cross_references", "bidirectional_text",
        ];
        for f in &feature_names {
            map.forbid(f.to_string());
        }
        SpaceProbe {
            map,
            forbidden: feature_names.iter().map(|s| s.to_string()).collect(),
            tested: Vec::new(),
        }
    }

    /// Mark a feature as tested.
    pub fn test_feature(&mut self, feature: &str) {
        self.map.occupy(feature.to_string(), true);
        if !self.tested.contains(&feature.to_string()) {
            self.tested.push(feature.to_string());
        }
    }

    /// Get features that remain untested (still in the forbidden zone).
    pub fn untested_features(&self) -> Vec<&str> {
        self.forbidden.iter()
            .filter(|f| !self.tested.contains(f))
            .map(|s| s.as_str())
            .collect()
    }

    /// Ratio of tested features (occupied space) to total features.
    pub fn coverage_ratio(&self) -> f64 {
        self.map.negative_space_ratio()
    }
}

impl Default for SpaceProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SpaceProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpaceProbe")
            .field("tested", &self.tested)
            .field("forbidden", &self.forbidden)
            .finish()
    }
}

/// Track formatting fidelity using conservation checking.
///
/// Ensures that key document quality metrics never decrease across parsing runs.
pub struct ConservationProbe {
    checker: ConservationChecker,
}

impl ConservationProbe {
    /// Create a new conservation probe with sensible defaults.
    pub fn new() -> Self {
        let mut checker = ConservationChecker::new();
        checker.register("table_cols", 100.0, 0.0);
        checker.register("table_rows", 100.0, 0.0);
        checker.register("text_chars", 10000.0, 50.0);
        checker.register("image_count", 10.0, 0.0);
        checker.register("header_count", 20.0, 0.0);
        ConservationProbe { checker }
    }

    /// Register a quantity that should be conserved.
    pub fn register(&mut self, name: &str, initial: f64, tolerance: f64) {
        self.checker.register(name, initial, tolerance);
    }

    /// Update a tracked quantity.
    pub fn update(&mut self, name: &str, value: f64) {
        self.checker.update(name, value);
    }

    /// Check if all tracked quantities are conserved (not decreased beyond tolerance).
    pub fn is_conserved(&self, name: &str) -> bool {
        self.checker.is_conserved(name)
    }

    /// Get any conservation violations.
    pub fn violations(&self) -> Vec<String> {
        self.checker.violations()
    }

    /// Take a snapshot for historical analysis.
    pub fn snapshot(&mut self) {
        self.checker.snapshot();
    }
}

impl Default for ConservationProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ConservationProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConservationProbe")
            .field("checker", &self.checker.check())
            .finish()
    }
}

/// Aggregate document parsing patterns using CracklePhase.
pub struct CrackleProbe {
    phase: CracklePhase<f64>,
}

impl Default for CrackleProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl CrackleProbe {
    /// Create a new crackle probe with standard statistical checks.
    pub fn new() -> Self {
        let phase = CracklePhase::<f64>::new()
            .on_cool("no_negative_variance", |vals| {
                if vals.is_empty() {
                    return true;
                }
                let mean = vals.iter().sum::<f64>() / vals.len() as f64;
                vals.iter().all(|v| *v >= mean * 0.5)
            })
            .on_cool("reasonable_spread", |vals| {
                if vals.len() < 2 {
                    return true;
                }
                let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
                (max - min) < 10000.0
            });
        CrackleProbe { phase }
    }

    /// Fire a data point (e.g., a quality metric from a parsed document).
    pub fn fire(&mut self, value: f64) {
        self.phase.fire(value);
    }

    /// Cool down and check patterns.
    pub fn cool(&mut self) -> bool {
        self.phase.cool().is_sound()
    }
}

impl std::fmt::Debug for CrackleProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrackleProbe").finish()
    }
}

/// Topological analysis of feature relationships using CathedralProbe.
pub struct FeatureGraphProbe {
    probe: CathedralProbe,
}

impl FeatureGraphProbe {
    /// Create a new feature graph probe for the given features.
    pub fn new(features: &[&str]) -> Self {
        let probe = CathedralProbe::new(features.to_vec());
        FeatureGraphProbe { probe }
    }

    /// Add a co-occurrence edge between two features with given weight.
    ///
    /// Weight should reflect how often these features appear together (0 = never, 1 = always).
    pub fn connect(&mut self, a: &str, b: &str, weight: f64) {
        self.probe.connect(a, b, weight);
    }

    /// Get the Fiedler value — how well-connected the feature graph is.
    /// Higher values mean features tend to co-occur more.
    pub fn fiedler_value(&self) -> f64 {
        self.probe.fiedler_value()
    }

    /// Check if the feature graph is healthy (well-connected).
    /// Unhealthy graphs (isolated feature clusters) indicate format silos.
    pub fn is_healthy(&self, threshold: f64) -> bool {
        self.probe.is_healthy(threshold)
    }

    /// Get the full eigenvalue spectrum.
    pub fn spectrum(&self) -> Vec<f64> {
        self.probe.spectrum()
    }
}

impl std::fmt::Debug for FeatureGraphProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureGraphProbe")
            .field("fiedler_value", &self.fiedler_value())
            .field("spectrum", &self.spectrum())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_probe() {
        let mut probe = SpaceProbe::new();
        assert_eq!(probe.untested_features().len(), 11);
        probe.test_feature("text");
        probe.test_feature("tables");
        assert_eq!(probe.untested_features().len(), 9);
    }

    #[test]
    fn test_conservation_probe() {
        let mut probe = ConservationProbe::new();
        // Initial is 10000 with tolerance 50, so 9950 is conserved
        probe.update("text_chars", 9950.0);
        assert!(probe.is_conserved("text_chars"));
        // Drop by 150 — exceeds 50 tolerance
        probe.update("text_chars", 9850.0);
        assert!(!probe.is_conserved("text_chars"));
    }

    #[test]
    fn test_crackle_probe() {
        let mut probe = CrackleProbe::new();
        probe.fire(100.0);
        probe.fire(110.0);
        probe.fire(95.0);
        assert!(probe.cool());
    }

    #[test]
    fn test_feature_graph_probe() {
        let features = vec!["text", "tables", "images"];
        let mut probe = FeatureGraphProbe::new(&features);
        probe.connect("text", "tables", 0.8);
        probe.connect("text", "images", 0.6);
        probe.connect("tables", "images", 0.3);
        assert!(probe.fiedler_value() > 0.0);
        assert!(probe.is_healthy(0.01));
        assert_eq!(probe.spectrum().len(), 3);
    }
}
