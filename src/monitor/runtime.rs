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

                // Anti-debug check
                match debugger.is_debugger_present() {
                    Ok(true) => {
                        dispatcher.dispatch(Event::DebuggerDetected);
                        let action = policy_engine.evaluate(&Event::DebuggerDetected);
                        dispatcher.dispatch(Event::PolicyAction {
                            event: "DebuggerDetected".into(),
                            action: format!("{:?}", action),
                        });
                    }
                    Ok(false) => {}
                    Err(e) => {
                        dispatcher.dispatch(Event::Error {
                            message: format!("debugger check failed: {}", e),
                        });
                    }
                }

                // Binary integrity check
                if let Some(ref binary) = binary_integrity {
                    match binary.verify_full() {
                        Ok(_) => {}
                        Err(_) => {
                            dispatcher.dispatch(Event::BinaryModified);
                            let action = policy_engine.evaluate(&Event::BinaryModified);
                            dispatcher.dispatch(Event::PolicyAction {
                                event: "BinaryModified".into(),
                                action: format!("{:?}", action),
                            });
                        }
                    }
                }

                // Library integrity check
                if let Some(ref library) = library_integrity {
                    match library.verify_all() {
                        Ok(mismatches) => {
                            if !mismatches.is_empty() {
                                dispatcher.dispatch(Event::LibraryModified);
                                let action = policy_engine.evaluate(&Event::LibraryModified);
                                dispatcher.dispatch(Event::PolicyAction {
                                    event: "LibraryModified".into(),
                                    action: format!("{:?}", action),
                                });
                            }
                        }
                        Err(e) => {
                            dispatcher.dispatch(Event::Error {
                                message: format!("library verification failed: {}", e),
                            });
                        }
                    }
                }

                // Memory integrity check
                if let Some(ref memory) = memory_integrity {
                    match memory.verify_all() {
                        Ok(modified) => {
                            if !modified.is_empty() {
                                dispatcher.dispatch(Event::MemoryIntegrityFailed);
                                let action = policy_engine.evaluate(&Event::MemoryIntegrityFailed);
                                dispatcher.dispatch(Event::PolicyAction {
                                    event: "MemoryIntegrityFailed".into(),
                                    action: format!("{:?}", action),
                                });
                            }
                        }
                        Err(e) => {
                            dispatcher.dispatch(Event::Error {
                                message: format!("memory verification failed: {}", e),
                            });
                        }
                    }
                }

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
