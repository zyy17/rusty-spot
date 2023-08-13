use std::future::Future;
use std::pin::Pin;

mod delay_future;
mod executor;
mod waker;

/// Provides functionality for delaying task execution.
pub use delay_future::DelayFuture;

/// An executor for handling and running asynchronous tasks.
pub use executor::MiniExecutor;

/// Utilities for waking up pending tasks.
pub use waker::MiniWaker;

/// A type alias for a future that can be used with the executor.
pub type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
