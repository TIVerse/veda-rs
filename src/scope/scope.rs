//! Scoped task spawning with lifetime guarantees.
//!
//! Scopes ensure that all spawned tasks complete before the scope exits,
//! allowing safe access to stack data.

use crossbeam_channel::{bounded, Sender, Receiver};
use std::marker::PhantomData;

/// A scope for spawning tasks that can access stack data.
pub struct Scope<'scope> {
    // Channel to track task completion
    tx: Sender<()>,
    rx: Receiver<()>,
    pending: usize,
    _marker: PhantomData<&'scope ()>,
}

impl<'scope> Scope<'scope> {
    fn new() -> Self {
        let (tx, rx) = bounded(1024);
        Self {
            tx,
            rx,
            pending: 0,
            _marker: PhantomData,
        }
    }
    
    /// Spawn a task within this scope.
    ///
    /// The task can safely access data from the enclosing scope because
    /// the scope guarantees all tasks complete before it exits.
    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        let tx = self.tx.clone();
        self.pending += 1;
        
        // SAFETY: The scope ensures this task completes before 'scope ends
        let f: Box<dyn FnOnce() + Send + 'static> = unsafe {
            std::mem::transmute(Box::new(f) as Box<dyn FnOnce() + Send + 'scope>)
        };
        
        crate::runtime::with_current_runtime(|rt| {
            rt.pool.execute(move || {
                f();
                let _ = tx.send(());
            });
        });
    }
}

impl<'scope> Drop for Scope<'scope> {
    fn drop(&mut self) {
        // Wait for all spawned tasks to complete
        for _ in 0..self.pending {
            let _ = self.rx.recv();
        }
    }
}

/// Execute a function with a scope for spawning scoped tasks.
///
/// # Examples
///
/// ```no_run
/// use veda::scope;
///
/// let mut data = vec![1, 2, 3, 4];
///
/// scope(|s| {
///     for elem in &mut data {
///         s.spawn(move || {
///             *elem *= 2;
///         });
///     }
/// });
///
/// assert_eq!(data, vec![2, 4, 6, 8]);
/// ```
pub fn scope<'scope, F, R>(f: F) -> R
where
    F: FnOnce(&mut Scope<'scope>) -> R,
{
    let mut scope = Scope::new();
    let result = f(&mut scope);
    drop(scope); // Explicit drop ensures all tasks complete
    result
}
