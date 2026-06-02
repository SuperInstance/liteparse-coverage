//! # Coverage Report
//!
//! The `CoverageReport` is the primary output of the topological analysis.
//! It aggregates Betti numbers, missing format families, persistent homology,
//! and actionable insights about what format combinations need testing.

use std::collections::{BTreeSet, HashMap, HashSet};
use crate::feature_space::{DocumentFeature, DocumentFeatureSpace};

/// A comprehensive report on document format coverage.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Betti numbers for the feature space simplicial complex.
    betti_numbers: Vec<usize>,
    /// Feature names in order.
    feature_labels: Vec<String>,
    /// Number of documents analyzed.
    total_documents: usize,
    /// Feature combinations that exist in tests.
    known_combinations: Vec<BTreeSet<String>>,
    /// Feature combinations missing from tests.
    missing_combinations: Vec<BTreeSet<String>>,
    /// Missing format families (grouped by dimension).
    missing_format_families: Vec<String>,
}

impl CoverageReport {
    /// Build a coverage report from a document feature space.
    pub(crate) fn new(space: &DocumentFeatureSpace) -> Self {
        let feature_labels = space.feature_labels();
        let betti_numbers = space.betti_numbers();

        // Collect known combinations as human-readable label sets
        let known_combinations: Vec<BTreeSet<String>> = space.simplices().iter()
            .map(|s| s.iter().map(|f| f.label().to_string()).collect())
            .collect();

        // Find missing combinations
        let missing = find_missing_combinations(space);

        // Format missing families
        let missing_format_families: Vec<String> = missing.iter()
            .map(|set| {
                let labels: Vec<&str> = set.iter().map(|s| s.as_str()).collect();
                format!("[{}]", labels.join(" + "))
            })
            .collect();

        CoverageReport {
            betti_numbers,
            feature_labels,
            total_documents: space.simplex_count(),
            known_combinations,
            missing_combinations: missing,
            missing_format_families,
        }
    }

    /// Betti numbers of the feature space.
    ///
    /// - Betti₀: connected components (format feature islands — should be 1 for full coverage)
    /// - Betti₁: 1-dimensional holes (untested pairs of features)
    /// - Betti₂: 2-dimensional holes (untested triples of features)
    /// - etc.
    pub fn betti_numbers(&self) -> &[usize] {
        &self.betti_numbers
    }

    /// Number of documents analyzed.
    pub fn total_documents(&self) -> usize {
        self.total_documents
    }

    /// Number of distinct features tracked.
    pub fn feature_count(&self) -> usize {
        self.feature_labels.len()
    }

    /// The feature labels.
    pub fn feature_labels(&self) -> &[String] {
        &self.feature_labels
    }

    /// Known feature combinations (the tested ones).
    pub fn known_combinations(&self) -> &[BTreeSet<String>] {
        &self.known_combinations
    }

    /// Missing format families — combinations of features that have never appeared together.
    pub fn missing_combinations(&self) -> &[BTreeSet<String>] {
        &self.missing_combinations
    }

    /// Human-readable list of missing format families.
    pub fn missing_format_families(&self) -> &[String] {
        &self.missing_format_families
    }

    /// Generate a human-readable summary of coverage.
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str("📊 Coverage Report\n");
        s.push_str("==================\n\n");
        s.push_str(&format!("Documents analyzed: {}\n", self.total_documents));
        s.push_str(&format!("Features tracked:   {}\n", self.feature_count()));
        s.push_str(&format!("Known combinations: {}\n", self.known_combinations.len()));
        s.push_str(&format!("Missing families:   {}\n\n", self.missing_format_families.len()));

        s.push_str("Betti Numbers (holes in coverage):\n");
        for (dim, &b) in self.betti_numbers.iter().enumerate() {
            let desc = match dim {
                0 => "connected components",
                1 => "1-dimensional holes (missing pairs)",
                2 => "2-dimensional holes (missing triples)",
                3 => "3-dimensional holes (missing quadruples)",
                _ => &format!("{}D holes", dim),
            };
            s.push_str(&format!("  β{} = {} — {}\n", dim, b, desc));
        }

        if !self.missing_format_families.is_empty() {
            s.push_str("\n⚠️  Missing format families:\n");
            for family in &self.missing_format_families {
                s.push_str(&format!("   ❌ {}\n", family));
            }
        }

        if self.betti_numbers.first().copied().unwrap_or(0) > 1 {
            s.push_str("\n🔴 HIGH Betti₀: Format features are tested in isolation.\n");
            s.push_str("   Combine features that are currently disconnected!\n");
        }

        s.push_str("\n💡 Recommendation: Aim for β₀ = 1, βᵢ = 0 for i ≥ 1.\n");
        s
    }
}

/// Find all feature combinations that are *not* present as simplices.
fn find_missing_combinations(space: &DocumentFeatureSpace) -> Vec<BTreeSet<String>> {
    let active_features = space.active_features();
    let feature_labels: HashMap<DocumentFeature, String> = active_features.iter()
        .map(|f| (*f, f.label().to_string()))
        .collect();

    let active_list: Vec<&DocumentFeature> = active_features.iter().collect();
    let n = active_list.len();
    if n < 2 {
        return vec![];
    }

    let existing_simplices: HashSet<BTreeSet<DocumentFeature>> = space.simplices().into_iter().collect();

    // Build the power set of active features (all possible combinations of size >= 2)
    // but only for combinations of size 2 and 3 (practical for reports)
    let mut missing = Vec::new();

    // Check pairs
    for i in 0..n {
        for j in (i + 1)..n {
            let mut pair = BTreeSet::new();
            pair.insert(*active_list[i]);
            pair.insert(*active_list[j]);

            // Check if this pair appears together in any known simplex
            let found = existing_simplices.iter().any(|s| pair.is_subset(s));
            if !found {
                let mut labeled = BTreeSet::new();
                labeled.insert(feature_labels[active_list[i]].clone());
                labeled.insert(feature_labels[active_list[j]].clone());
                missing.push(labeled);
            }
        }
    }

    // Check triples (only those where all sub-pairs are known but the triple isn't)
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let mut triple = BTreeSet::new();
                triple.insert(*active_list[i]);
                triple.insert(*active_list[j]);
                triple.insert(*active_list[k]);

                let found = existing_simplices.iter().any(|s| triple.is_subset(s));
                if !found {
                    let mut labeled = BTreeSet::new();
                    labeled.insert(feature_labels[active_list[i]].clone());
                    labeled.insert(feature_labels[active_list[j]].clone());
                    labeled.insert(feature_labels[active_list[k]].clone());
                    missing.push(labeled);
                }
            }
        }
    }

    // Sort: pairs first, then triples, alphabetical within groups
    missing.sort_by(|a, b| {
        let a_len = a.len();
        let b_len = b.len();
        a_len.cmp(&b_len).then_with(|| {
            let a_str: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
            let b_str: Vec<&str> = b.iter().map(|s| s.as_str()).collect();
            a_str.join(",").cmp(&b_str.join(","))
        })
    });

    missing
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_empty_space() {
        let space = DocumentFeatureSpace::new();
        let report = CoverageReport::new(&space);
        assert_eq!(report.total_documents(), 0);
    }

    #[test]
    fn test_report_with_data() {
        let mut space = DocumentFeatureSpace::new();
        space.add_document("a", &["text", "tables"]);
        space.add_document("b", &["text", "images"]);
        // "tables + images" is a missing pair
        let report = CoverageReport::new(&space);
        assert_eq!(report.total_documents(), 2);
        assert!(!report.missing_format_families().is_empty());
        assert!(report.missing_format_families().iter().any(|f| f.contains("tables") && f.contains("images")));
    }

    #[test]
    fn test_summary_generation() {
        let mut space = DocumentFeatureSpace::new();
        space.add_document("basic", &["text"]);
        space.add_document("with_tables", &["text", "tables"]);
        let report = CoverageReport::new(&space);
        let summary = report.summary();
        assert!(summary.contains("Coverage Report"));
        assert!(summary.contains("Betti Numbers"));
    }
}
