//! Error types for the VEDA runtime.

/// Result type alias for VEDA operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the VEDA runtime.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Scheduler-related error
    #[error("scheduler error: {0}")]
    Scheduler(String),

    /// Executor error
    #[error("executor error: {0}")]
    Executor(String),

    /// Configuration error
    #[error("configuration error: {0}")]
    Config(String),

    /// Runtime not initialized
    #[error("runtime not initialized - call veda::init() first")]
    NotInitialized,

    /// Runtime already initialized
    #[error("runtime already initialized")]
    AlreadyInitialized,

    /// Worker thread panic
    #[error("worker thread panicked: {0}")]
    WorkerPanic(String),

    /// Task execution error
    #[error("task execution failed: {0}")]
    TaskFailed(String),

    /// GPU-related error
    #[cfg(feature = "gpu")]
    #[error("GPU error: {0}")]
    Gpu(String),

    /// Async runtime error
    #[cfg(feature = "async")]
    #[error("async error: {0}")]
    Async(String),

    /// NUMA error
    #[cfg(feature = "numa")]
    #[error("NUMA error: {0}")]
    Numa(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error for future extensibility
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a scheduler error
    pub fn scheduler<S: Into<String>>(msg: S) -> Self {
        Error::Scheduler(msg.into())
    }

    /// Create an executor error
    pub fn executor<S: Into<String>>(msg: S) -> Self {
        Error::Executor(msg.into())
    }

    /// Create a config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Error::Config(msg.into())
    }
    
    /// Create a GPU error
    #[cfg(feature = "gpu")]
    pub fn gpu<S: Into<String>>(msg: S) -> Self {
        Error::Gpu(msg.into())
    }
    
    /// Create an async error
    #[cfg(feature = "async")]
    pub fn async_error<S: Into<String>>(msg: S) -> Self {
        Error::Async(msg.into())
    }
    
    /// Create a telemetry error (maps to Other)
    #[cfg(feature = "telemetry")]
    pub fn telemetry<S: Into<String>>(msg: S) -> Self {
        Error::Other(format!("telemetry: {}", msg.into()))
    }
}
