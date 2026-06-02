# liteparse-coverage 🔬📄

> **liteparse parses documents. negative-space-testing finds what it can't. Together: a parser that knows its own boundaries.**

[![Crates.io](https://img.shields.io/crates/v/liteparse-coverage.svg)](https://crates.io/crates/liteparse-coverage)
[![Documentation](https://docs.rs/liteparse-coverage/badge.svg)](https://docs.rs/liteparse-coverage)

**Topological coverage analysis** for [liteparse](https://github.com/run-llama/liteparse) (9K ★, fast open-source document parser from LlamaIndex) using [negative-space-testing](https://github.com/SuperInstance/negative-space-testing).

Instead of just knowing what you've tested, this crate shows you **what you haven't tested** — the holes in your parser's format coverage — using **topological data analysis** (simplicial complexes, Betti numbers, persistent homology).

---

## The Problem

Your document parser supports 11+ format features: text, tables, images, nested lists, formulas, code blocks, headers, footnotes, cross-references, bidirectional text, lists. But *combinations* of these features create a combinatorial explosion:

- 11 features → **2^11 = 2,048 possible combinations**
- Most teams test 10-20 documents → **< 1% coverage**
- You don't know which format families are untested until a user reports a bug

**The insight:** The untested combinations form *holes* in a mathematical space. We find those holes using topology.

---

## How It Works

### 1. Build a Feature Space

Each parsed document becomes a **simplex** — a set of features it contains:

```
Document A: {text, tables}
Document B: {text, images}
Document C: {text, tables, images, formulas}
```

This builds a **simplicial complex** — a topological space where known-working configurations live.

### 2. Compute Betti Numbers

Betti numbers count the **holes** in this space:

| Symbol | Name | What it means for coverage |
|--------|------|---------------------------|
| **β₀** | Connected components | **Format silos** — if > 1, some features are tested in complete isolation |
| **β₁** | 1-dimensional holes | **Missing pairs** — two features that have never appeared together in a test |
| **β₂** | 2-dimensional holes | **Missing triples** — three features that have never co-occurred |
| **βₙ** | n-dimensional holes | Missing combinations of n+1 features |

### 3. Persistent Homology

Track *when* holes appear and disappear as features are progressively added. This tells you which feature combinations are the "most missing" — the ones that would fill the most topological holes.

### 4. Coverage Report

```rust
use liteparse_coverage::DocumentFeatureSpace;

let mut space = DocumentFeatureSpace::new();
space.add_document("basic_report", &["text"]);
space.add_document("report_with_table", &["text", "tables"]);
space.add_document("report_with_image", &["text", "images"]);

let report = space.analyze();

println!("{}", report.summary());
// 📊 Coverage Report
// ==================
// Documents analyzed: 3
// Features tracked:   3
// Betti₀ = 1 — one connected component
// Betti₁ = 1 — missing pair: tables + images
// ⚠️  Missing format family: [tables + images]
```

---

## Getting Started

```bash
cargo add liteparse-coverage
```

### Basic Usage

```rust
use liteparse_coverage::{
    DocumentFeatureSpace,
    CoverageProbe,
    DocumentSignature,
    CoverageReport,
};

// Build a feature space
let mut space = DocumentFeatureSpace::new();

// Add known-working document configurations
space.add_document("simple_memo", &["text", "headers"]);
space.add_document("data_report", &["text", "tables", "headers"]);
space.add_document("presentation", &["text", "images", "headers"]);
space.add_document("scientific_paper", &["text", "tables", "footnotes", "formulas"]);
space.add_document("readme", &["text", "code_blocks", "lists", "headers"]);

// Analyze coverage holes
let report = space.analyze();
println!("{}", report.summary());

// What's missing?
for family in report.missing_format_families() {
    println!("❌ Untested: {}", family);
}
```

### Working with Parsed Documents

When using the real `liteparse` crate, you can convert `ParsedPage` output to `DocumentSignature`:

```rust
use liteparse_coverage::DocumentSignature;
use liteparse::ParsedPage;  // liteparse-integration feature

fn analyze_parsed_doc(pages: &[ParsedPage]) -> DocumentSignature {
    let mut sig = DocumentSignature::default();
    for page in pages {
        for item in &page.items {
            sig.has_text = true;  // detect text items
            // ... detect other features
        }
    }
    sig
}
```

### Probe Parsed Output

Use `CoverageProbe` with `negative-space-testing` to check for forbidden patterns:

```rust
use liteparse_coverage::CoverageProbe;

let probe = CoverageProbe::new();
let content = "Extracted table: Name | Age\nJohn | 30";  // no errors
assert!(probe.check(content).is_clean());
```

---

## Integration

This crate is designed to be used alongside `liteparse`. Enable the `liteparse-integration` feature for tighter coupling:

```toml
[dependencies]
liteparse-coverage = { version = "0.1", features = ["liteparse-integration"] }
```

---

## The Math (Briefly)

### Simplicial Complex

A set of features `{f₁, f₂, ..., fₙ}` forms an *(n-1)*-simplex. The collection of all simplices forms a simplicial complex *K*.

### Chain Complex

For each dimension *k*, *C_k* is the vector space (over ℤ₂) generated by *k*-simplices. The boundary map ∂ₖ: *C_k → C_{k-1}* sends a *k*-simplex to its *(k-1)*-faces.

### Homology

```
H_k(K) = ker(∂ₖ) / im(∂_{k+1})
βₖ = dim(Hₖ)
```

Betti number βₖ counts the *k*-dimensional holes in the feature space — feature combinations of size *k+1* that are untested.

### Persistent Homology

Track homology classes through a filtration:

```
∅ = K₀ ⊆ K₁ ⊆ K₂ ⊆ ... ⊆ Kₘ = K
```

Features that persist longer are more structurally significant — they represent fundamental gaps in format coverage.

---

## License

MIT OR Apache-2.0
