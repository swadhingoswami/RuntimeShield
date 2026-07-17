use crate::events::Event;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

pub type EventCallback = Arc<dyn Fn(Event) + Send + Sync>;

pub struct EventDispatcher {
    callbacks: Arc<Mutex<Vec<EventCallback>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register(&mut self, callback: EventCallback) {
        if let Ok(mut guard) = self.callbacks.lock() {
            guard.push(callback);
        }
    }

    pub fn dispatch(&self, event: Event) {
        if let Ok(guard) = self.callbacks.lock() {
            for cb in guard.iter() {
                let cb = cb.clone();
                let ev = event.clone();
                let _ = panic::catch_unwind(AssertUnwindSafe(move || {
                    cb(ev);
                }));
            }
        }
    }

    pub fn clone_dispatcher(&self) -> Self {
        Self {
            callbacks: self.callbacks.clone(),
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_dispatch() {
        let mut dispatcher = EventDispatcher::new();
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        dispatcher.register(Arc::new(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        }));

        dispatcher.dispatch(Event::VerificationStarted);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_callbacks() {
        let mut dispatcher = EventDispatcher::new();
        let results = Arc::new(Mutex::new(Vec::new()));

        for i in 0..3 {
            let results = results.clone();
            dispatcher.register(Arc::new(move |e| {
                results.lock().unwrap().push((i, e));
            }));
        }

        dispatcher.dispatch(Event::VerificationCompleted);
        assert_eq!(results.lock().unwrap().len(), 3);
    }
}
