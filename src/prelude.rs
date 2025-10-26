//! Convenient re-exports for common VEDA types and traits.
//!
//! This module provides a single import for most common use cases:
//! ```
//! use veda::prelude::*;
//! ```

pub use crate::config::{Config, ConfigBuilder, SchedulingPolicy};
pub use crate::error::{Error, Result};
pub use crate::iter::{IntoParallelIterator, ParallelIterator};

// Re-export spawn and other top-level functions when they're defined
#[doc(inline)]
pub use crate::{init, init_with_config};

// Scope support
#[doc(inline)]
pub use crate::scope::scope;
