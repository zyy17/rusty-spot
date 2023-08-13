use std::time::Duration;

use rusty_spot::{DelayFuture, MiniExecutor};

// A simple asynchronous task that will print a message after a delay.
async fn task_one() {
    let delay = DelayFuture::new(Duration::from_secs(2));
    delay.await;
    println!("Task one completed after 2 seconds!");
}

// Another asynchronous task that will print a message after a shorter delay.
async fn task_two() {
    let delay = DelayFuture::new(Duration::from_secs(1));
    delay.await;
    println!("Task two completed after 1 second!");
}

fn main() {
    // Create a new instance of the MiniExecutor.
    let executor = MiniExecutor::new();

    // Spawn our asynchronous tasks.
    executor.spawn(task_one());
    executor.spawn(task_two());

    // Run the executor.
    executor.run();
}
