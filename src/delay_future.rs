use futures::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

/// A future that completes after a specified duration.
pub struct DelayFuture {
    duration: Duration,

    // Indicates if the background thread was started.
    started: AtomicBool,

    // Indicates if the specified duration has elapsed.
    completed: Arc<AtomicBool>,
}

impl DelayFuture {
    /// Creates a new `DelayFuture` that completes after the given duration.
    pub fn new(duration: Duration) -> Self {
        DelayFuture {
            duration,
            started: AtomicBool::new(false),
            completed: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Future for DelayFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if the duration has elapsed.
        if self.completed.load(Ordering::Relaxed) {
            return Poll::Ready(());
        }

        // If not started, spawn a background thread to wait for the specified duration.
        if !self.started.swap(true, Ordering::Relaxed) {
            let completed = self.completed.clone();
            let waker = cx.waker().clone();

            // Move `self.duration` into a local variable so it can be safely used within the spawned thread.
            let duration = self.duration;

            thread::spawn(move || {
                thread::sleep(duration);
                completed.store(true, Ordering::Relaxed);
                waker.wake_by_ref(); // Notify the executor that the future can make progress.
            });
        }

        // If the duration has not elapsed yet, return Pending.
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn it_delays_correctly() {
        let mut future = DelayFuture::new(Duration::from_millis(100));

        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        // Initially, the future should be pending.
        assert_eq!(Pin::new(&mut future).poll(&mut cx), Poll::Pending);

        let start = Instant::now();
        thread::sleep(Duration::from_millis(150));

        // After the sleep in the main thread, the future should be ready.
        assert_eq!(Pin::new(&mut future).poll(&mut cx), Poll::Ready(()));
        assert!(start.elapsed() >= Duration::from_millis(100));
    }
}
