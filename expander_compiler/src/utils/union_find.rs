//! Union-Find data structure for efficient disjoint set operations.

/// A simple Union-Find (Disjoint Set Union) implementation.
pub struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    /// Creates a new Union-Find structure with `n` elements, each element is its own parent.
    pub fn new(n: usize) -> Self {
        let parent = (0..n).collect();
        Self { parent }
    }

    /// Finds the root of the set containing `x`, applying path compression.
    pub fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    /// Unites the sets containing `x` and `y`.
    pub fn union(&mut self, x: usize, y: usize) {
        let x = self.find(x);
        let y = self.find(y);
        self.parent[x] = y;
    }
}
