use std::collections::VecDeque;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use crate::Task;
use crate::waker::MiniWaker;

/// `MiniExecutor` represents a simple task executor for running asynchronous tasks.
/// This executor will continuously poll tasks in its queue and execute them.
pub struct MiniExecutor {
    tasks: Arc<Mutex<VecDeque<Task>>>,
}

impl MiniExecutor {
    /// Creates a new MiniExecutor with an empty tasks queue.
    pub fn new() -> Self {
        MiniExecutor {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push_back(Box::pin(future));
    }

    /// Runs the executor, processing each task in the queue until all tasks have been completed.
    pub fn run(&self) {
        // Create a waker for the executor.
        let waker = Arc::new(MiniWaker::new());
        let std_waker = MiniWaker::get_waker(&waker);

        loop {
            let mut tasks = self.tasks.lock().unwrap();
            if tasks.is_empty() {
                break;
            }
            let mut task = tasks.pop_front().unwrap();
            let mut cx = Context::from_waker(&std_waker);
            match task.as_mut().poll(&mut cx) {
                // If the task is not yet ready, push it back onto the queue.
                Poll::Pending => {
                    tasks.push_back(task);
                }

                // If it's done, do nothing (just drop the task).
                Poll::Ready(()) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::thread;
    use std::time::Duration;

    use crate::delay_future::DelayFuture;

    #[test]
    fn test_executor_with_delay() {
        let executor = MiniExecutor::new();

        executor.spawn(async {
            // Simulate some asynchronous work using a delay.
            let delay_future = DelayFuture::new(Duration::from_millis(10));
            delay_future.await;
        });

        executor.spawn(async {
            // Simulate some asynchronous work using a delay.
            let delay_future = DelayFuture::new(Duration::from_millis(20));
            delay_future.await;
        });

        thread::spawn(move || {
            // Run the executor.
            executor.run();
        });
    }
}
