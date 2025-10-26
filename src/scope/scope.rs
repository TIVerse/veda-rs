use crossbeam_channel::{bounded, Receiver, Sender};
use std::marker::PhantomData;

pub struct Scope<'scope> {
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

    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        let tx = self.tx.clone();
        self.pending += 1;

        let f: Box<dyn FnOnce() + Send + 'static> =
            unsafe { std::mem::transmute(Box::new(f) as Box<dyn FnOnce() + Send + 'scope>) };

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
        for _ in 0..self.pending {
            let _ = self.rx.recv();
        }
    }
}

pub fn scope<'scope, F, R>(f: F) -> R
where
    F: FnOnce(&mut Scope<'scope>) -> R,
{
    let mut scope = Scope::new();
    let result = f(&mut scope);
    drop(scope);
    result
}
