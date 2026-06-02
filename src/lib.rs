//! # liteparse-coverage
//!
//! Topological coverage analysis for [liteparse](https://github.com/run-llama/liteparse)
//! using [negative-space-testing](https://github.com/SuperInstance/negative-space-testing).
//!
//! **liteparse parses documents. negative-space-testing finds what it can't.**
//! **Together: a parser that knows its own boundaries.**
//!
//! This crate builds a **document feature space** — a simplicial complex where each vertex
//! is a document feature (tables, images, nested lists, formulas, code blocks) and each
//! simplex represents a known-working combination of features. Using topological data
//! analysis, we compute **Betti numbers** to find "holes" in parser format coverage.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use liteparse_coverage::{DocumentFeatureSpace, CoverageReport};
//!
//! // Build a feature space from known working document configurations
//! let mut space = DocumentFeatureSpace::new();
//! space.add_document("basic_report", &["text"]);
//! space.add_document("report_with_table", &["text", "table"]);
//! space.add_document("report_with_image", &["text", "image"]);
//! space.add_document("report_with_table_and_image", &["text", "table", "image"]);
//!
//! // Find holes in coverage
//! let report = space.analyze();
//!
//! // Which format combinations are untested?
//! let holes = report.missing_format_families();
//! for hole in holes {
//!     println!("Missing format family: {}", hole);
//! }
//! ```

mod feature_space;
mod coverage;
pub mod probes;
pub mod report;

pub use feature_space::DocumentFeatureSpace;
pub use coverage::{CoverageProbe, DocumentSignature};
pub use report::CoverageReport;

// Re-export negative-space-testing for convenience
pub use negative_space_testing as nst;
