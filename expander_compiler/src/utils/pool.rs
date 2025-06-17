//! A simple pool implementation for storing unique values with their indices.
//! This pool allows adding values, retrieving their indices, and accessing the values by index.

use std::{collections::HashMap, hash::Hash};

/// The `Pool` struct is a generic container that holds a vector of values and a mapping from values to their indices.
/// When a value is added, it is stored in the vector and its index is recorded in the map.
/// If the value already exists in the pool, its existing index is returned.
#[derive(Default, Clone)]
pub struct Pool<V> {
    vec: Vec<V>,
    map: HashMap<V, usize>,
}

impl<V> Pool<V>
where
    V: Hash + Eq + Clone,
{
    /// Creates a new empty `Pool`.
    pub fn new() -> Self {
        Pool {
            vec: Vec::new(),
            map: HashMap::new(),
        }
    }

    /// Adds a value to the pool, returning its index.
    /// If the value already exists, it returns the existing index.
    pub fn add(&mut self, v: &V) -> usize {
        if let Some(&idx) = self.map.get(v) {
            return idx;
        }
        let idx = self.vec.len();
        self.vec.push(v.clone());
        self.map.insert(v.clone(), idx);
        idx
    }

    /// Gets the index of a value in the pool.
    pub fn get_idx(&self, val: &V) -> usize {
        *self.map.get(val).expect("Pool value does not exist")
    }

    /// Tries to get the index of a value in the pool, returning `None` if the value does not exist.
    pub fn try_get_idx(&self, val: &V) -> Option<usize> {
        self.map.get(val).cloned()
    }

    /// Retrieves a reference to the value at the specified index.
    pub fn get(&self, idx: usize) -> &V {
        self.vec.get(idx).expect("Pool index out of bounds")
    }

    /// Retrieves a reference to the value at the specified index.
    pub fn vec(&self) -> &Vec<V> {
        &self.vec
    }

    /// Retrieves a reference to the internal map that associates values with their indices.
    pub fn map(&self) -> &HashMap<V, usize> {
        &self.map
    }

    /// Returns the number of values in the pool.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Checks if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}
