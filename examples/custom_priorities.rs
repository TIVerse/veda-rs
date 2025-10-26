//! Custom task priorities example

use veda::prelude::*;
use veda::scope;
use veda::scheduler::priority::Priority;
use std::time::Instant;

fn main() {
    println!("=== Custom Task Priorities Example ===\n");
    
    veda::init().expect("Failed to initialize VEDA");
    
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
    
    veda::shutdown();
    println!("\n=== Example Complete ===");
}
