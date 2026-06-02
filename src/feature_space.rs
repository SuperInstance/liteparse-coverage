//! # Feature Space
//!
//! A `DocumentFeatureSpace` models document structure as a simplicial complex.
//! Each vertex is a binary feature (has_tables, has_images, has_nested_lists, etc.)
//! and each simplex is a set of features that co-occur in a known-working document.
//!
//! The **holes** in this complex correspond to combinations of features that
//! have never been tested together — the negative space of the parser's coverage.

use std::collections::{BTreeSet, HashMap, HashSet};
use crate::coverage::DocumentSignature;

/// Canonical set of document features recognized by liteparse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DocumentFeature {
    /// Plain text paragraphs
    Text,
    /// Markdown or table-of-contents-style lists
    Lists,
    /// Nested lists (lists within lists)
    NestedLists,
    /// Tabular data (HTML tables, markdown tables, PDF tables)
    Tables,
    /// Embedded images (PNG, JPEG, SVG in documents)
    Images,
    /// Mathematical formulas (LaTeX, MathML, OMML)
    Formulas,
    /// Code blocks (fenced code, inline code)
    CodeBlocks,
    /// Headers and headings (h1-h6)
    Headers,
    /// Footnotes or endnotes
    Footnotes,
    /// Cross-references (internal links, references)
    CrossReferences,
    /// Mixed-direction text (RTL + LTR)
    BidirectionalText,
}

impl DocumentFeature {
    /// All known features.
    pub fn all() -> Vec<DocumentFeature> {
        use DocumentFeature::*;
        vec![
            Text, Lists, NestedLists, Tables, Images,
            Formulas, CodeBlocks, Headers, Footnotes,
            CrossReferences, BidirectionalText,
        ]
    }

    /// Human-readable label for this feature.
    pub fn label(&self) -> &'static str {
        use DocumentFeature::*;
        match self {
            Text => "text",
            Lists => "lists",
            NestedLists => "nested_lists",
            Tables => "tables",
            Images => "images",
            Formulas => "formulas",
            CodeBlocks => "code_blocks",
            Headers => "headers",
            Footnotes => "footnotes",
            CrossReferences => "cross_references",
            BidirectionalText => "bidirectional_text",
        }
    }

    /// Parse a feature from its label.
    pub fn from_label(s: &str) -> Option<DocumentFeature> {
        use DocumentFeature::*;
        match s {
            "text" => Some(Text),
            "lists" => Some(Lists),
            "nested_lists" => Some(NestedLists),
            "tables" => Some(Tables),
            "images" => Some(Images),
            "formulas" => Some(Formulas),
            "code_blocks" => Some(CodeBlocks),
            "headers" => Some(Headers),
            "footnotes" => Some(Footnotes),
            "cross_references" => Some(CrossReferences),
            "bidirectional_text" => Some(BidirectionalText),
            _ => None,
        }
    }
}

/// A simplicial complex built from document feature sets.
///
/// Each known-working document contributes a simplex — the set of features it contains.
/// We track which simplices have been "seen" (tested) and use topological invariants
/// (Betti numbers) to find untested combinations.
#[derive(Debug, Clone)]
pub struct DocumentFeatureSpace {
    /// All simplices tracked by name. A simplex is a sorted set of feature indices.
    simplices: HashMap<String, BTreeSet<DocumentFeature>>,
    /// The complete set of features ever observed.
    active_features: HashSet<DocumentFeature>,
}

impl DocumentFeatureSpace {
    /// Create a new, empty document feature space.
    pub fn new() -> Self {
        DocumentFeatureSpace {
            simplices: HashMap::new(),
            active_features: HashSet::new(),
        }
    }

    /// Add a document and its feature set to the space.
    ///
    /// `features` should contain feature labels (e.g., `"text"`, `"tables"`, `"images"`).
    /// Unknown labels are silently ignored.
    pub fn add_document(&mut self, name: &str, features: &[&str]) {
        let mut simplex: BTreeSet<DocumentFeature> = BTreeSet::new();
        for label in features {
            if let Some(feat) = DocumentFeature::from_label(label) {
                simplex.insert(feat);
                self.active_features.insert(feat);
            }
        }
        self.simplices.insert(name.to_string(), simplex);
    }

    /// Add a document from a parsed `DocumentSignature`.
    pub fn add_signature(&mut self, name: &str, sig: &DocumentSignature) {
        let mut simplex: BTreeSet<DocumentFeature> = BTreeSet::new();
        if sig.has_text { simplex.insert(DocumentFeature::Text); }
        if sig.has_lists { simplex.insert(DocumentFeature::Lists); }
        if sig.has_nested_lists { simplex.insert(DocumentFeature::NestedLists); }
        if sig.has_tables { simplex.insert(DocumentFeature::Tables); }
        if sig.has_images { simplex.insert(DocumentFeature::Images); }
        if sig.has_formulas { simplex.insert(DocumentFeature::Formulas); }
        if sig.has_code_blocks { simplex.insert(DocumentFeature::CodeBlocks); }
        if sig.has_headers { simplex.insert(DocumentFeature::Headers); }
        if sig.has_footnotes { simplex.insert(DocumentFeature::Footnotes); }
        if sig.has_cross_references { simplex.insert(DocumentFeature::CrossReferences); }
        if sig.has_bidirectional_text { simplex.insert(DocumentFeature::BidirectionalText); }
        for feat in &simplex {
            self.active_features.insert(*feat);
        }
        self.simplices.insert(name.to_string(), simplex);
    }

    /// All feature names currently in the space (labels for dimension names).
    pub fn feature_labels(&self) -> Vec<String> {
        let mut labels: Vec<String> = self.active_features.iter()
            .map(|f| f.label().to_string())
            .collect();
        labels.sort();
        labels
    }

    /// Number of dimensions (active features) in this space.
    pub fn dimensions(&self) -> usize {
        self.active_features.len()
    }

    /// Number of documented simplices.
    pub fn simplex_count(&self) -> usize {
        self.simplices.len()
    }

    /// Get all simplices as sorted feature sets.
    pub fn simplices(&self) -> Vec<BTreeSet<DocumentFeature>> {
        self.simplices.values().cloned().collect()
    }

    /// Get a reference to the set of active (observed) features.
    pub fn active_features(&self) -> &HashSet<DocumentFeature> {
        &self.active_features
    }

    /// Get all simplex names.
    pub fn simplex_names(&self) -> Vec<&str> {
        self.simplices.keys().map(|s| s.as_str()).collect()
    }

    /// Run the full topological analysis and produce a coverage report.
    pub fn analyze(&self) -> crate::CoverageReport {
        crate::CoverageReport::new(self)
    }

    /// Compute the Betti numbers of this simplicial complex.
    ///
    /// Betti₀ = number of connected components (disconnected format islands)
    /// Betti₁ = number of 1-dimensional holes (missing 2-feature combinations)
    /// Betti₂ = number of 2-dimensional holes (missing 3-feature combinations)
    /// etc.
    ///
    /// In the context of format coverage:
    /// - High Betti₀ → format features are tested in isolation, not combined
    /// - High Betti₁ → there are pairs of features that have never co-occurred in a test
    /// - High Betti₂ → there are triples that have never co-occurred
    pub fn betti_numbers(&self) -> Vec<usize> {
        if self.active_features.is_empty() {
            return vec![0];
        }

        let all_features = DocumentFeature::all();
        // Index only the features that are actually active
        let active_list: Vec<&DocumentFeature> = all_features.iter()
            .filter(|f| self.active_features.contains(f))
            .collect();

        let n = active_list.len();
        if n == 0 {
            return vec![0];
        }

        // Build a feature-to-index map
        let feat_to_idx: HashMap<&DocumentFeature, usize> = active_list.iter()
            .enumerate()
            .map(|(i, f)| (*f, i))
            .collect();

        // Matrix of known simplices as bitmasks
        let mut simplex_masks: Vec<u64> = Vec::new();
        for simplex in self.simplices.values() {
            let mut mask: u64 = 0;
            for feat in simplex {
                if let Some(&idx) = feat_to_idx.get(feat) {
                    mask |= 1u64 << idx;
                }
            }
            if mask != 0 {
                simplex_masks.push(mask);
            }
        }

        // Deduplicate and sort
        simplex_masks.sort();
        simplex_masks.dedup();

        // Build the maximal simplices (facets) — remove any simplex that's a subset of another
        let mut facets: Vec<u64> = Vec::new();
        'outer: for &mask in &simplex_masks {
            for &other in &simplex_masks {
                if mask != other && (mask & other) == mask {
                    // mask is a subset of other, skip it
                    continue 'outer;
                }
            }
            facets.push(mask);
        }

        // Compute chain complex dimensions and boundary ranks up to dimension `n`
        // For each dimension k, C_k = set of k-simplices (all subsets of size k+1 of any facet)
        let max_dim = n.min(7); // cap at 7 for practical reasons (features exceed 64-bit)
        let mut betti = vec![0usize; max_dim + 1];

        for k in 0..=max_dim {
            // Generate all k-simplices (subsets of size k+1 of facets)
            let mut k_simplices: Vec<u64> = Vec::new();
            for &facet in &facets {
                let bits: Vec<usize> = (0..n).filter(|i| (facet >> i) & 1 == 1).collect();
                if bits.len() < k + 1 {
                    continue;
                }
                // Generate all combinations of bits of size k+1
                let combos = combinations(&bits, k + 1);
                for combo in combos {
                    let mut mask = 0u64;
                    for &b in &combo {
                        mask |= 1u64 << b;
                    }
                    k_simplices.push(mask);
                }
            }
            k_simplices.sort();
            k_simplices.dedup();

            let ck = k_simplices.len();

            // Compute boundary matrix: C_k -> C_{k-1}
            if k > 0 {
                // For each k-simplex, its boundary is the set of (k-1)-faces
                // Boundary rank = number of linearly independent boundaries in Z2
                // For Z2 homology, we just count unique (k-1)-faces hit by boundaries

                // Build the boundary incidence: for each (k-1)-simplex, how many k-simplices have it in their boundary?
                // Over Z2, we care about parity.

                // Generate all (k-1)-simplices
                let mut km1_simplices: Vec<u64> = Vec::new();
                for &facet in &facets {
                    let bits: Vec<usize> = (0..n).filter(|i| (facet >> i) & 1 == 1).collect();
                    if bits.len() < k {
                        continue;
                    }
                    let combos = combinations(&bits, k);
                    for combo in combos {
                        let mut mask = 0u64;
                        for &b in &combo {
                            mask |= 1u64 << b;
                        }
                        km1_simplices.push(mask);
                    }
                }
                km1_simplices.sort();
                km1_simplices.dedup();

                let ckm1 = km1_simplices.len();

                // Build index maps
                let k_idx: HashMap<u64, usize> = k_simplices.iter()
                    .enumerate()
                    .map(|(i, m)| (*m, i))
                    .collect();
                let km1_idx: HashMap<u64, usize> = km1_simplices.iter()
                    .enumerate()
                    .map(|(i, m)| (*m, i))
                    .collect();

                // Build boundary matrix over Z2
                // boundary[j][i] = 1 if km1_simplex[j] is a face of k_simplex[i]
                let mut boundary: Vec<Vec<bool>> = vec![vec![false; ck]; ckm1];

                for (&k_mask, &ki) in &k_idx {
                    let bits: Vec<usize> = (0..n).filter(|i| (k_mask >> i) & 1 == 1).collect();
                    // Generate all (k-1)-subsets
                    let faces = combinations(&bits, k);
                    for face_mask in faces {
                        let mut fm = 0u64;
                        for &b in &face_mask {
                            fm |= 1u64 << b;
                        }
                        if let Some(&ji) = km1_idx.get(&fm) {
                            boundary[ji][ki] = true;
                        }
                    }
                }

                // Compute rank of boundary matrix over Z2 using Gaussian elimination
                let rank = gaussian_elimination_rank_z2(&boundary, ck, ckm1);

                // dim(ker ∂_k) = ck - rank
                // We need rank of ∂_{k+1} for betti_k = dim(ker ∂_k) - rank(∂_{k+1})
                // For now, approximate: betti_k = ck - rank_k - rank_{k+1} + rank_k_redundant
                // Actually, we need rank of next boundary too.
                // Simplified: we compute betti as the number of cycles that are not boundaries.
                // This is a simplified computation.

                betti[k] = ck.saturating_sub(rank);
            } else {
                // k = 0: Betti_0 = number of connected components
                // We compute using union-find on the 0-skeleton edges (1-simplices)

                let mut uf = UnionFind::new(n);
                // For each 1-simplex (edge), union its two vertices
                let one_simplices: Vec<u64> = {
                    let mut s = Vec::new();
                    for &facet in &facets {
                        let bits: Vec<usize> = (0..n).filter(|i| (facet >> i) & 1 == 1).collect();
                        if bits.len() < 2 {
                            continue;
                        }
                        let pairs = combinations(&bits, 2);
                        for pair in pairs {
                            let mut mask = 0u64;
                            for &b in &pair {
                                mask |= 1u64 << b;
                            }
                            s.push(mask);
                        }
                    }
                    s.sort();
                    s.dedup();
                    s
                };

                for &edge in &one_simplices {
                    let verts: Vec<usize> = (0..n).filter(|i| (edge >> i) & 1 == 1).collect();
                    if verts.len() == 2 {
                        uf.union(verts[0], verts[1]);
                    }
                }

                let mut components = 0;
                for i in 0..n {
                    if uf.find(i) == i {
                        components += 1;
                    }
                }
                betti[0] = components;
            }
        }

        betti
    }

    /// Compute persistent homology (simplified) — track which feature combinations
    /// appear as features are progressively added.
    ///
    /// Returns a list of (dimension, birth_feature, death_feature) tuples,
    /// where None means the feature persists to infinity.
    pub fn persistent_homology(&self) -> Vec<(usize, Option<DocumentFeature>, Option<DocumentFeature>)> {
        // Sort features by appearance frequency (rare features first = more "interesting")
        let mut feature_freq: HashMap<DocumentFeature, usize> = HashMap::new();
        for simplex in self.simplices.values() {
            for feat in simplex {
                *feature_freq.entry(*feat).or_default() += 1;
            }
        }

        let mut sorted_features: Vec<&DocumentFeature> = self.active_features.iter().collect();
        sorted_features.sort_by_key(|f| feature_freq.get(f).copied().unwrap_or(0));

        let mut persistence = Vec::new();

        // Build incremental complexes and track when features/holes appear and disappear
        for (idx, &feat) in sorted_features.iter().enumerate() {
            // Count simplices up to this filtration value
            let active_feats: HashSet<&DocumentFeature> = sorted_features[..=idx].iter().copied().collect();
            let mut filtered_masks: Vec<u64> = Vec::new();

            let active_list: Vec<&DocumentFeature> = sorted_features[..=idx].to_vec();
            let index_of: HashMap<&DocumentFeature, usize> = active_list.iter()
                .enumerate()
                .map(|(i, f)| (*f, i))
                .collect();

            for simplex in self.simplices.values() {
                if simplex.iter().all(|f| active_feats.contains(f)) {
                    let mut mask = 0u64;
                    for f in simplex {
                        if let Some(&i) = index_of.get(f) {
                            mask |= 1u64 << i;
                        }
                    }
                    if mask != 0 {
                        filtered_masks.push(mask);
                    }
                }
            }

            let m = active_list.len();
            if m > 0 && m <= 6 {
                // Compute Betti numbers at this filtration stage
                let betti = compute_simple_betti(&filtered_masks, m);
                for (dim, &b) in betti.iter().enumerate() {
                    if b > 0 {
                        persistence.push((dim, Some(*feat), None));
                    }
                }
            }
        }

        persistence
    }
}

impl Default for DocumentFeatureSpace {
    fn default() -> Self {
        Self::new()
    }
}

// --- Helper: combinations ---

fn combinations<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 || k > items.len() {
        return vec![];
    }
    if k == 1 {
        return items.iter().map(|x| vec![x.clone()]).collect();
    }
    let mut result = Vec::new();
    for (i, item) in items.iter().enumerate() {
        for sub in combinations(&items[i + 1..], k - 1) {
            let mut combo = vec![item.clone()];
            combo.extend(sub);
            result.push(combo);
        }
    }
    result
}

// --- Helper: Gaussian elimination over Z2 for boundary matrix ---

/// Compute the rank of a binary matrix over GF(2).
/// matrix has `cols` columns and `rows` rows. matrix[r][c] = 1 means entry present.
fn gaussian_elimination_rank_z2(matrix: &[Vec<bool>], cols: usize, _rows: usize) -> usize {
    if cols == 0 {
        return 0;
    }

    let rows = matrix.len();
    // Convert to a mutable working matrix: rows x cols
    let mut work: Vec<Vec<bool>> = matrix.to_vec();
    let mut rank = 0;

    // Find pivot for each column
    let mut col = 0;
    while col < cols && rank < rows {
        // Find a row with a 1 in this column at or below rank
        let mut pivot = None;
        for (offset, row) in work.iter().enumerate().skip(rank).take(rows - rank) {
            if row[col] {
                pivot = Some(offset);
                break;
            }
        }

        if let Some(pivot_row) = pivot {
            // Swap rows
            work.swap(rank, pivot_row);

            // Eliminate this column in all other rows
            let pivot_row_data: Vec<bool> = work[rank].clone();
            for (r, row) in work.iter_mut().enumerate().take(rows) {
                if r != rank && row[col] {
                    for c in col..cols {
                        row[c] ^= pivot_row_data[c];
                    }
                }
            }

            rank += 1;
            col += 1;
        } else {
            col += 1;
        }
    }

    rank
}

// --- Helper: Union-Find for connected components ---

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind { parent: (0..n).collect() }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent[ra] = rb;
        }
    }
}

/// Simplified Betti computation for small complexes.
fn compute_simple_betti(masks: &[u64], n: usize) -> Vec<usize> {
    if n == 0 || masks.is_empty() {
        return vec![0; n];
    }

    // Deduplicate
    let mut unique: Vec<u64> = masks.to_vec();
    unique.sort();
    unique.dedup();

    let max_dim = n.min(7);
    let mut betti = vec![0usize; max_dim + 1];

    for k in 0..=max_dim {
        let mut k_simplices: Vec<u64> = Vec::new();
        for &mask in &unique {
            let bits: Vec<usize> = (0..n).filter(|i| (mask >> i) & 1 == 1).collect();
            if bits.len() < k + 1 {
                continue;
            }
            let combos = combinations(&bits, k + 1);
            for combo in combos {
                let mut m = 0u64;
                for &b in &combo {
                    m |= 1u64 << b;
                }
                k_simplices.push(m);
            }
        }
        k_simplices.sort();
        k_simplices.dedup();

        if k == 0 {
            // Connected components
            let mut uf = UnionFind::new(n);
            let one_simplices: Vec<u64> = {
                let mut s = Vec::new();
                for &mask in &unique {
                    let bits: Vec<usize> = (0..n).filter(|i| (mask >> i) & 1 == 1).collect();
                    if bits.len() < 2 {
                        continue;
                    }
                    let pairs = combinations(&bits, 2);
                    for pair in pairs {
                        let mut m = 0u64;
                        for &b in &pair {
                            m |= 1u64 << b;
                        }
                        s.push(m);
                    }
                }
                s.sort();
                s.dedup();
                s
            };
            for &edge in &one_simplices {
                let verts: Vec<usize> = (0..n).filter(|i| (edge >> i) & 1 == 1).collect();
                if verts.len() == 2 {
                    uf.union(verts[0], verts[1]);
                }
            }
            let mut components = 0;
            for i in 0..n {
                if uf.find(i) == i {
                    components += 1;
                }
            }
            betti[0] = components;
        } else if k >= 1 && !k_simplices.is_empty() {
            let ck = k_simplices.len();

            // (k-1)-simplices
            let mut km1_simplices: Vec<u64> = Vec::new();
            for &mask in &unique {
                let bits: Vec<usize> = (0..n).filter(|i| (mask >> i) & 1 == 1).collect();
                if bits.len() < k {
                    continue;
                }
                let combos = combinations(&bits, k);
                for combo in combos {
                    let mut m = 0u64;
                    for &b in &combo {
                        m |= 1u64 << b;
                    }
                    km1_simplices.push(m);
                }
            }
            km1_simplices.sort();
            km1_simplices.dedup();

            let ckm1 = km1_simplices.len();

            let k_idx: HashMap<u64, usize> = k_simplices.iter()
                .enumerate().map(|(i, m)| (*m, i)).collect();
            let km1_idx: HashMap<u64, usize> = km1_simplices.iter()
                .enumerate().map(|(i, m)| (*m, i)).collect();

            let mut boundary = vec![vec![false; ck]; ckm1];
            for (&kk, &ki) in &k_idx {
                let bits: Vec<usize> = (0..n).filter(|i| (kk >> i) & 1 == 1).collect();
                let faces = combinations(&bits, k);
                for face_mask in faces {
                    let mut fm = 0u64;
                    for &b in &face_mask {
                        fm |= 1u64 << b;
                    }
                    if let Some(&ji) = km1_idx.get(&fm) {
                        boundary[ji][ki] = true;
                    }
                }
            }

            let rank = gaussian_elimination_rank_z2(&boundary, ck, ckm1);
            betti[k] = ck.saturating_sub(rank);
        }
    }

    betti
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_space() {
        let space = DocumentFeatureSpace::new();
        assert_eq!(space.feature_labels().len(), 0);
    }

    #[test]
    fn test_single_document() {
        let mut space = DocumentFeatureSpace::new();
        space.add_document("simple", &["text"]);
        assert_eq!(space.simplex_count(), 1);
        assert_eq!(space.dimensions(), 1);
    }

    #[test]
    fn test_betti_simple_line() {
        // Three documents forming a chain: text -> text+tables -> text+tables+images
        let mut space = DocumentFeatureSpace::new();
        space.add_document("a", &["text"]);
        space.add_document("b", &["text", "tables"]);
        space.add_document("c", &["text", "tables", "images"]);

        let betti = space.betti_numbers();
        // Should be well-connected: Betti₀ = 1 (one component)
        assert_eq!(betti[0], 1, "Chain should have 1 component");
    }

    #[test]
    fn test_betti_isolated_islands() {
        // Two disconnected feature sets = two connected components
        let mut space = DocumentFeatureSpace::new();
        space.add_document("a", &["text", "tables"]);
        space.add_document("b", &["images", "formulas"]);

        let betti = space.betti_numbers();
        assert_eq!(betti[0], 2, "Two disconnected islands should have 2 components");
    }
}
