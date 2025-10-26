pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("scheduler error: {0}")]
    Scheduler(String),

    #[error("executor error: {0}")]
    Executor(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("runtime not initialized")]
    NotInitialized,

    #[error("already initialized")]
    AlreadyInitialized,

    #[error("worker panic: {0}")]
    WorkerPanic(String),

    #[error("task failed: {0}")]
    TaskFailed(String),

    #[cfg(feature = "gpu")]
    #[error("GPU error: {0}")]
    Gpu(String),

    #[cfg(feature = "async")]
    #[error("async error: {0}")]
    Async(String),

    #[cfg(feature = "numa")]
    #[error("NUMA error: {0}")]
    Numa(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn scheduler<S: Into<String>>(msg: S) -> Self {
        Error::Scheduler(msg.into())
    }

    pub fn executor<S: Into<String>>(msg: S) -> Self {
        Error::Executor(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        Error::Config(msg.into())
    }

    #[cfg(feature = "gpu")]
    pub fn gpu<S: Into<String>>(msg: S) -> Self {
        Error::Gpu(msg.into())
    }

    #[cfg(feature = "async")]
    pub fn async_error<S: Into<String>>(msg: S) -> Self {
        Error::Async(msg.into())
    }

    #[cfg(feature = "telemetry")]
    pub fn telemetry<S: Into<String>>(msg: S) -> Self {
        Error::Other(format!("telemetry: {}", msg.into()))
    }
}
