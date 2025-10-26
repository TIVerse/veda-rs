//! Scoped parallelism example - safe task spawning with lifetimes

use veda::prelude::*;
use veda::scope;
use std::sync::Arc;
use parking_lot::Mutex;

fn main() {
    println!("=== Scoped Parallelism Example ===\n");
    
    veda::init().expect("Failed to initialize VEDA");
    
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
            let mut total = self.value;
            
            scope::scope(|s| {
                let results: Vec<_> = self.children.iter()
                    .map(|child| {
                        s.spawn(move || child.sum_parallel())
                    })
                    .collect();
                
                for handle in results {
                    total += handle.join().unwrap_or(0);
                }
            });
            
            total
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
    
    veda::shutdown();
    println!("\n=== Example Complete ===");
}
