//! Parallel iterator types and traits.
//!
//! This module provides Rayon-compatible parallel iterators for VEDA.

pub mod par_iter;

pub use par_iter::{IntoParallelIterator, ParallelIterator};
