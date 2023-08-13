use futures::task::{waker_ref, ArcWake};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A simple waker implementation that can be used to determine if
/// a future has requested to be woken up.
pub struct MiniWaker {
    is_woken: AtomicBool,
}

impl MiniWaker {
    /// Creates a new `MiniWaker`.
    pub fn new() -> Self {
        MiniWaker {
            is_woken: AtomicBool::new(false),
        }
    }

    /// Check if the waker has been woken up.
    pub fn is_woken(&self) -> bool {
        self.is_woken.load(Ordering::Relaxed)
    }

    /// Get a standard Waker for the given MiniWaker.
    pub fn get_waker(arc_self: &Arc<Self>) -> std::task::Waker {
        waker_ref(arc_self).clone()
    }
}

impl ArcWake for MiniWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Use the underlying MiniWaker directly without trying to consume the Arc.
        arc_self.is_woken.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::task::Context;
    use std::thread;
    use std::time::Duration;

    use futures::Future;

    use crate::delay_future::DelayFuture;

    #[test]
    fn test_waker() {
        let waker = Arc::new(MiniWaker::new());
        let std_waker = MiniWaker::get_waker(&waker);

        // Initially, the waker has not been woken.
        assert_eq!(waker.is_woken(), false);

        // Wake the waker.
        std_waker.wake_by_ref();
        //
        // The waker should now report that it's been woken.
        assert_eq!(waker.is_woken(), true);
    }

    #[test]
    fn test_with_delay_future() {
        let waker = Arc::new(MiniWaker::new());
        let std_waker = MiniWaker::get_waker(&waker);
        let mut delay_future = Box::pin(DelayFuture::new(Duration::from_millis(100)));

        // Context for polling the future
        let mut cx = Context::from_waker(&std_waker);

        // Before waiting for the duration, it should be pending
        assert!(matches!(
            delay_future.as_mut().poll(&mut cx),
            std::task::Poll::Pending
        ));

        // Wait for slightly more than the delay duration
        thread::sleep(Duration::from_millis(150));

        // After the delay, the future should be ready, but you need to wake it first.
        std_waker.wake_by_ref();
        assert!(matches!(
            delay_future.as_mut().poll(&mut cx),
            std::task::Poll::Ready(())
        ));
    }
}
