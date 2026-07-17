use crate::core::error::Result;
use crate::events::callback::EventDispatcher;
use crate::events::Event;
use crate::integrity::binary::BinaryIntegrity;
use crate::integrity::library::LibraryIntegrity;
use crate::integrity::memory::MemoryIntegrity;
use crate::platform::{DebuggerDetector, PlatformDebugger};
use crate::policy::engine::PolicyEngine;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct RuntimeMonitor {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl RuntimeMonitor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
}

impl Default for RuntimeMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeMonitor {

    pub fn start(
        &mut self,
        interval_ms: u64,
        binary_integrity: Option<BinaryIntegrity>,
        library_integrity: Option<LibraryIntegrity>,
        memory_integrity: Option<MemoryIntegrity>,
        dispatcher: EventDispatcher,
        policy_engine: PolicyEngine,
    ) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let debugger = PlatformDebugger::new();

        self.handle = Some(thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                dispatcher.dispatch(Event::VerificationStarted);

                check_debugger(&debugger, &dispatcher, &policy_engine);
                check_binary(&binary_integrity, &dispatcher, &policy_engine);
                check_libraries(&library_integrity, &dispatcher, &policy_engine);
                check_memory(&memory_integrity, &dispatcher, &policy_engine);

                dispatcher.dispatch(Event::VerificationCompleted);

                thread::sleep(Duration::from_millis(interval_ms));
            }
        }));

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RuntimeMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

fn handle_policy(
    event: Event,
    action_event: &str,
    dispatcher: &EventDispatcher,
    policy_engine: &PolicyEngine,
) {
    let action = policy_engine.evaluate(&event);
    dispatcher.dispatch(Event::PolicyAction {
        event: action_event.into(),
        action: format!("{:?}", action),
    });
    if policy_engine.execute(&action, &event) {
        std::process::exit(1);
    }
}

fn check_debugger(
    debugger: &PlatformDebugger,
    dispatcher: &EventDispatcher,
    policy_engine: &PolicyEngine,
) {
    match debugger.is_debugger_present() {
        Ok(true) => {
            handle_policy(Event::DebuggerDetected, "DebuggerDetected", dispatcher, policy_engine);
        }
        Ok(false) => {}
        Err(e) => {
            dispatcher.dispatch(Event::Error {
                message: format!("debugger check failed: {}", e),
            });
        }
    }
}

fn check_binary(
    binary_integrity: &Option<BinaryIntegrity>,
    dispatcher: &EventDispatcher,
    policy_engine: &PolicyEngine,
) {
    if let Some(ref binary) = binary_integrity {
        match binary.verify_full() {
            Ok(_) => {}
            Err(crate::core::error::Error::HashMismatch { expected, actual }) => {
                dispatcher.dispatch(Event::HashMismatch { expected, actual });
                handle_policy(Event::BinaryModified, "BinaryModified", dispatcher, policy_engine);
            }
            Err(e) => {
                dispatcher.dispatch(Event::Error {
                    message: format!("binary verification failed: {}", e),
                });
            }
        }
    }
}

fn check_libraries(
    library_integrity: &Option<LibraryIntegrity>,
    dispatcher: &EventDispatcher,
    policy_engine: &PolicyEngine,
) {
    if let Some(ref library) = library_integrity {
        match library.verify_all() {
            Ok(mismatches) => {
                if !mismatches.is_empty() {
                    handle_policy(Event::LibraryModified, "LibraryModified", dispatcher, policy_engine);
                }
            }
            Err(e) => {
                dispatcher.dispatch(Event::Error {
                    message: format!("library verification failed: {}", e),
                });
            }
        }
    }
}

fn check_memory(
    memory_integrity: &Option<MemoryIntegrity>,
    dispatcher: &EventDispatcher,
    policy_engine: &PolicyEngine,
) {
    if let Some(ref memory) = memory_integrity {
        match memory.verify_all() {
            Ok(modified) => {
                if !modified.is_empty() {
                    handle_policy(Event::MemoryIntegrityFailed, "MemoryIntegrityFailed", dispatcher, policy_engine);
                }
            }
            Err(e) => {
                dispatcher.dispatch(Event::Error {
                    message: format!("memory verification failed: {}", e),
                });
            }
        }
    }
}
