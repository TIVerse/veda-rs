//! Scoped parallelism example - safe task spawning with lifetimes

use veda_rs::prelude::*;
use veda_rs::scope;
use std::sync::Arc;
use parking_lot::Mutex;

fn main() {
    println!("=== Scoped Parallelism Example ===\n");
    
    veda_rs::init().expect("Failed to initialize VEDA");
    
    // Example 1: Mutable access to local data
    let mut data = vec![0; 100];
    let data_mutex = Arc::new(Mutex::new(&mut data));
    
    println!("Incrementing array elements in parallel...");
    
    scope::scope(|s| {
        for i in 0..10 {
            let data_ref = data_mutex.clone();
            s.spawn(move || {
                let mut guard = data_ref.lock();
                for j in 0..10 {
                    guard[i * 10 + j] += 1;
                }
            });
        }
    });
    
    println!("Sum after parallel increment: {}", data.iter().sum::<i32>());
    
    // Example 2: Parallel tree traversal
    #[derive(Debug)]
    struct Node {
        value: i32,
        children: Vec<Node>,
    }
    
    impl Node {
        fn new(value: i32) -> Self {
            Self { value, children: Vec::new() }
        }
        
        fn with_children(value: i32, children: Vec<Node>) -> Self {
            Self { value, children }
        }
        
        fn sum_parallel(&self) -> i32 {
            use std::sync::Arc;
            use parking_lot::Mutex;
            
            let total = Arc::new(Mutex::new(self.value));
            
            scope::scope(|s| {
                for child in &self.children {
                    let total_ref = total.clone();
                    let child_sum = child.sum_parallel();
                    s.spawn(move || {
                        *total_ref.lock() += child_sum;
                    });
                }
            });
            
            Arc::try_unwrap(total).unwrap().into_inner()
        }
    }
    
    println!("\nBuilding tree and computing sum...");
    
    let tree = Node::with_children(
        1,
        vec![
            Node::with_children(2, vec![Node::new(4), Node::new(5)]),
            Node::with_children(3, vec![Node::new(6), Node::new(7)]),
        ],
    );
    
    let sum = tree.sum_parallel();
    println!("Tree sum: {} (expected: 28)", sum);
    
    veda_rs::shutdown();
    println!("\n=== Example Complete ===");
}
