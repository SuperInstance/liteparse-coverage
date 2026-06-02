//! # Coverage Analysis
//!
//! The `CoverageProbe` uses `negative-space-testing`'s `NegativeTest` to validate that
//! parsed documents never contain certain forbidden patterns, while the `DocumentFeatureSpace`
//! finds holes in the positive space of known-working format combinations.
//!
//! For a real liteparse integration, you'd provide actual parsed documents. This module
//! defines the `DocumentSignature` trait for extracting feature vectors from parsed docs.


/// A signature extracted from a parsed document, indicating which features it contains.
#[derive(Debug, Clone, Default)]
pub struct DocumentSignature {
    pub has_text: bool,
    pub has_lists: bool,
    pub has_nested_lists: bool,
    pub has_tables: bool,
    pub has_images: bool,
    pub has_formulas: bool,
    pub has_code_blocks: bool,
    pub has_headers: bool,
    pub has_footnotes: bool,
    pub has_cross_references: bool,
    pub has_bidirectional_text: bool,
}

impl DocumentSignature {
    /// Create a signature from a feature label set.
    pub fn from_labels(labels: &[&str]) -> Self {
        let mut sig = DocumentSignature::default();
        for &label in labels {
            match label {
                "text" => sig.has_text = true,
                "lists" => sig.has_lists = true,
                "nested_lists" => sig.has_nested_lists = true,
                "tables" => sig.has_tables = true,
                "images" => sig.has_images = true,
                "formulas" => sig.has_formulas = true,
                "code_blocks" => sig.has_code_blocks = true,
                "headers" => sig.has_headers = true,
                "footnotes" => sig.has_footnotes = true,
                "cross_references" => sig.has_cross_references = true,
                "bidirectional_text" => sig.has_bidirectional_text = true,
                _ => {}
            }
        }
        sig
    }

    /// Number of features present in this signature.
    pub fn feature_count(&self) -> usize {
        let mut count = 0;
        if self.has_text { count += 1; }
        if self.has_lists { count += 1; }
        if self.has_nested_lists { count += 1; }
        if self.has_tables { count += 1; }
        if self.has_images { count += 1; }
        if self.has_formulas { count += 1; }
        if self.has_code_blocks { count += 1; }
        if self.has_headers { count += 1; }
        if self.has_footnotes { count += 1; }
        if self.has_cross_references { count += 1; }
        if self.has_bidirectional_text { count += 1; }
        count
    }

    /// Convert to a vector of feature labels.
    pub fn to_labels(&self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if self.has_text { labels.push("text"); }
        if self.has_lists { labels.push("lists"); }
        if self.has_nested_lists { labels.push("nested_lists"); }
        if self.has_tables { labels.push("tables"); }
        if self.has_images { labels.push("images"); }
        if self.has_formulas { labels.push("formulas"); }
        if self.has_code_blocks { labels.push("code_blocks"); }
        if self.has_headers { labels.push("headers"); }
        if self.has_footnotes { labels.push("footnotes"); }
        if self.has_cross_references { labels.push("cross_references"); }
        if self.has_bidirectional_text { labels.push("bidirectional_text"); }
        labels
    }
}

/// A probe that uses `negative-space-testing` to validate parsed documents.
///
/// `CoverageProbe` wraps a `NegativeTest` that checks parsed documents for
/// forbidden conditions (e.g., missing output, garbled tables, lost images).
#[derive(Debug)]
pub struct CoverageProbe {
    /// The NegativeTest validator — stored as a builder, checked at runtime
    forbidden_names: Vec<String>,
}

impl CoverageProbe {
    /// Create a new coverage probe with standard forbid rules.
    pub fn new() -> Self {
        CoverageProbe {
            forbidden_names: vec![
                "empty_output".to_string(),
                "parse_error_in_output".to_string(),
                "garbled_tables".to_string(),
                "ocr_failure_marker".to_string(),
                "no_text_extracted".to_string(),
            ],
        }
    }

    /// Build the internal NegativeTest from our rules.
    fn build_test(&self) -> negative_space_testing::NegativeTest<String> {
        use negative_space_testing::NegativeTest;
        let mut test = NegativeTest::<String>::new();
        for name in &self.forbidden_names {
            let n = name.clone();
            match n.as_str() {
                "empty_output" => {
                    test = test.forbid("empty_output", move |s| s.trim().is_empty());
                }
                "parse_error_in_output" => {
                    test = test.forbid("parse_error_in_output", move |s| {
                        s.contains("ERROR") || s.contains("ParseError")
                    });
                }
                "garbled_tables" => {
                    test = test.forbid("garbled_tables", move |s| s.contains("[GARBLED]"));
                }
                "ocr_failure_marker" => {
                    test = test.forbid("ocr_failure_marker", move |s| s.contains("[OCR_FAILED]"));
                }
                "no_text_extracted" => {
                    test = test.forbid("no_text_extracted", move |s| s.trim().len() < 5);
                }
                _ => {}
            }
        }
        test
    }

    /// Check a parsed document string for forbidden patterns.
    pub fn check(&self, content: &str) -> negative_space_testing::NegativeResult {
        self.build_test().check(&content.to_string())
    }

    /// Check multiple parsed document strings.
    pub fn check_all(&self, contents: &[&str]) -> negative_space_testing::NegativeResult {
        let strings: Vec<String> = contents.iter().map(|s| s.to_string()).collect();
        self.build_test().check_all(&strings)
    }
}

impl Default for CoverageProbe {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_from_labels() {
        let sig = DocumentSignature::from_labels(&["text", "tables", "images"]);
        assert!(sig.has_text);
        assert!(sig.has_tables);
        assert!(sig.has_images);
        assert!(!sig.has_code_blocks);
        assert_eq!(sig.feature_count(), 3);
    }

    #[test]
    fn test_coverage_probe_clean() {
        let probe = CoverageProbe::new();
        let result = probe.check("This is a perfectly parsed document with some content.");
        assert!(result.is_clean());
    }

    #[test]
    fn test_coverage_probe_violation() {
        let probe = CoverageProbe::new();
        let result = probe.check("");
        assert!(!result.is_clean());
        assert!(result.violations.iter().any(|v| v == "empty_output"));
    }

    #[test]
    fn test_coverage_probe_parse_error() {
        let probe = CoverageProbe::new();
        let result = probe.check("Some content before ERROR: ParseError in table extraction");
        assert!(!result.is_clean());
        assert!(result.violations.iter().any(|v| v == "parse_error_in_output"));
    }
}
