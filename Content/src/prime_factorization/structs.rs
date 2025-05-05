use super::Context;

use rug::{Assign, Integer};

/// Fixed-size array of (Integer, usize, Context) with length tracking
/// All elements are pre-initialized with (Integer::ONE, 0, Context(Integer::ONE))
#[derive(Clone, Debug)]
pub struct FixedVec<T, const N: usize> {
    pub data: [T; N],
    pub length: usize,
}

impl<T: Clone, const N: usize> FixedVec<T, N> {
    /// Creates a new array with all elements cloned from a template
    pub fn new(template: T) -> Self {
        let data = std::array::from_fn(|_| template.clone());
        Self { data, length: 0 }
    }

    /// call this before assigning a new item into the array
    pub fn inc(&mut self) {
        // assert!(self.length < N, "FixedVec overflow");
        self.length += 1;
    }
    
    /// call this after "removing" the back from the array
    pub fn dec(&mut self) {
        self.length -= 1;
    }

    /// Returns immutable reference to the element at index
    pub fn get(&self, index: usize) -> &T {
        &self.data[index]
    }

    /// Returns mutable reference to the element at index
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index]
    }

    /// Returns the last pushed element
    pub fn top(&mut self) -> &mut T {
        &mut self.data[self.length - 1]
    }

    /// Returns the last pushed element
    pub fn next(&mut self) -> &mut T {
        &mut self.data[self.length]
    }

    /// Returns current number of elements
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns true if empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// swaps 2 entries
    pub fn swap(&mut self, a: usize, b: usize) {
        self.data.swap(a, b);
    }

    /// Clears the vector (does not reset the values)
    pub fn clear(&mut self) {
        self.length = 0;
    }
}

#[derive(Clone, Debug)]
pub struct Factor {
    pub n: Integer,
    pub idx: usize,
    pub ctx: Context,
}

impl Factor {
    pub fn new() -> Self {
        let n = Integer::new();
        let idx = 0;
        let ctx = Context::new(Integer::ONE.clone());
        Factor { n, idx, ctx }
    }
    
    /// Changes the factor with the given n and index, and updates the context
    pub fn update_all(&mut self, n: &Integer, idx: usize) {
        self.n.assign(n);
        self.idx = idx;
        self.ctx.change_mod(n);
    }

    /// Updates the context with the current n value
    pub fn update_ctx(&mut self) {
        self.ctx.change_mod(&self.n);
    }
    
    /// Updates n and the index. this does NOT update the context
    pub fn update_n_and_index(&mut self, n: &Integer, idx: usize) {
        self.idx = idx;
        self.n.assign(n);
    }

    /// Assigns the value of another factor to this one
    pub fn assign(&mut self, other: &Factor) {
        self.n.assign(&other.n);
        self.idx = other.idx;
        self.ctx.assign(&other.ctx);
    }
}