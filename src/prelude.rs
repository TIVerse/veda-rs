pub use crate::config::{Config, ConfigBuilder, SchedulingPolicy};
pub use crate::error::{Error, Result};
pub use crate::iter::{IntoParallelIterator, ParallelIterator, ParallelSlice};
pub use crate::executor::Priority;
pub use crate::runtime_manager::RuntimeManager;

pub use crate::{init, init_with_config, shutdown};
pub use crate::scope::scope;

#[cfg(feature = "telemetry")]
pub use crate::telemetry::{Metrics, MetricsSnapshot};

#[cfg(feature = "async")]
pub use crate::async_bridge::{spawn_async, block_on, ParStreamExt};
