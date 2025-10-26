//! Custom task priorities example

use std::time::Instant;
use veda_rs::prelude::*;
use veda_rs::scheduler::priority::Priority;
use veda_rs::scope;

fn main() {
    println!("=== Custom Task Priorities Example ===\n");

    veda_rs::init().expect("Failed to initialize VEDA");

    println!("Spawning tasks with different priorities...");

    scope::scope(|s| {
        // Low priority background tasks
        for i in 0..5 {
            s.spawn(move || {
                println!("[LOW] Background task {} starting", i);
                std::thread::sleep(std::time::Duration::from_millis(100));
                println!("[LOW] Background task {} complete", i);
            });
        }

        // High priority tasks
        for i in 0..3 {
            s.spawn(move || {
                println!("[HIGH] Priority task {} starting", i);
                std::thread::sleep(std::time::Duration::from_millis(50));
                println!("[HIGH] Priority task {} complete", i);
            });
        }

        // Normal priority tasks
        for i in 0..5 {
            s.spawn(move || {
                println!("[NORMAL] Regular task {} starting", i);
                std::thread::sleep(std::time::Duration::from_millis(75));
                println!("[NORMAL] Regular task {} complete", i);
            });
        }
    });

    println!("\nAll tasks completed!");

    veda_rs::shutdown();
    println!("\n=== Example Complete ===");
}
