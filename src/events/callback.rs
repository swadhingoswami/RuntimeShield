use crate::events::Event;
use std::sync::Arc;

pub type EventCallback = Arc<dyn Fn(Event) + Send + Sync>;

pub struct EventDispatcher {
    callbacks: Vec<EventCallback>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn register(&mut self, callback: EventCallback) {
        self.callbacks.push(callback);
    }

    pub fn dispatch(&self, event: Event) {
        for cb in &self.callbacks {
            cb(event.clone());
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
        let results = Arc::new(std::sync::Mutex::new(Vec::new()));

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
