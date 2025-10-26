pub mod par_iter;
pub mod advanced_combinators;
pub mod chunks;

pub use par_iter::{IntoParallelIterator, ParallelIterator, ParallelSlice};
pub use advanced_combinators::{FlatMap, Zip};
pub use chunks::{Chunks, Windows, ParallelChunks};
