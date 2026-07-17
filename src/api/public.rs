use crate::config::policy::PolicyConfig;
use crate::core::error::Result;
use crate::events::callback::{EventCallback, EventDispatcher};
use crate::events::Event;
use crate::integrity::binary::BinaryIntegrity;
use crate::integrity::library::LibraryIntegrity;
use crate::integrity::memory::MemoryIntegrity;
use crate::core::builder::RuntimeShieldBuilder;
use crate::monitor::runtime::RuntimeMonitor;
use crate::platform::{PlatformDebugger, DebuggerDetector, PlatformProcess, ProcessIdentity};
use crate::policy::engine::PolicyEngine;

pub struct RuntimeShield {
    builder: RuntimeShieldBuilder,
    policy: Option<PolicyConfig>,
    policy_engine: Option<PolicyEngine>,
    dispatcher: EventDispatcher,
    monitor: Option<RuntimeMonitor>,
    binary_integrity: Option<BinaryIntegrity>,
    library_integrity: Option<LibraryIntegrity>,
    memory_integrity: Option<MemoryIntegrity>,
    started: bool,
}

impl RuntimeShield {
    pub fn new(builder: RuntimeShieldBuilder, policy: Option<PolicyConfig>) -> Self {
        let dispatcher = EventDispatcher::new();
        Self {
            builder,
            policy,
            policy_engine: None,
            dispatcher,
            monitor: None,
            binary_integrity: None,
            library_integrity: None,
            memory_integrity: None,
            started: false,
        }
    }

    pub fn builder() -> RuntimeShieldBuilder {
        RuntimeShieldBuilder::new()
    }

    pub fn start(&mut self) -> Result<()> {
        if self.started {
            return Ok(());
        }

        let policy = self.policy.clone().unwrap_or_default();
        let policy_engine = PolicyEngine::new(policy);
        self.policy_engine = Some(policy_engine);

        if let Some(ref cb) = self.builder.callback {
            self.dispatcher.register(cb.clone());
        }

        self.dispatcher.dispatch(Event::Info {
            message: "RuntimeShield starting".into(),
        });

        // Share the same dispatcher with the background thread via Arc<Mutex>
        // Any on_event() calls after start() will also fire in the monitor
        let shared_dispatcher = self.dispatcher.clone_dispatcher();

        // Startup verification
        if self.builder.startup_verification {
            self.startup_verification()?;
        }

        // Initialize binary integrity
        if self.builder.binary_integrity {
            let proc = PlatformProcess::new();
            let exe_path = proc.executable_path()?;
            let mut bin = BinaryIntegrity::new(&exe_path);

            if let Some(ref manifest_path) = self.builder.manifest_path {
                bin.load_manifest_from_path(std::path::Path::new(manifest_path))?;
            }

            self.binary_integrity = Some(bin);
        }

        // Initialize library integrity
        if self.builder.library_integrity {
            let mut lib = LibraryIntegrity::new();
            lib.enumerate_loaded_libraries()?;
            self.library_integrity = Some(lib);
        }

        // Initialize memory integrity
        if self.builder.memory_integrity {
            let mut mem = MemoryIntegrity::new();
            mem.auto_discover_code_regions()?;
            mem.snapshot_hashes()?;
            self.memory_integrity = Some(mem);
        }

        // Start runtime monitor
        if self.builder.runtime_monitor {
            let mut monitor = RuntimeMonitor::new();
            let monitor_policy = self.policy.clone().unwrap_or_default();
            let monitor_policy_engine = PolicyEngine::new(monitor_policy);

            monitor.start(
                self.builder.monitor_interval_ms,
                self.binary_integrity.clone(),
                self.library_integrity.clone(),
                self.memory_integrity.clone(),
                shared_dispatcher,
                monitor_policy_engine,
            )?;
            self.monitor = Some(monitor);
        }

        self.started = true;

        self.dispatcher.dispatch(Event::Info {
            message: "RuntimeShield started".into(),
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(ref mut monitor) = self.monitor {
            monitor.stop();
        }
        self.started = false;

        self.dispatcher.dispatch(Event::Info {
            message: "RuntimeShield stopped".into(),
        });
    }

    pub fn verify_now(&self) -> Result<VerificationResult> {
        let mut result = VerificationResult::new();

        if let Some(ref binary) = self.binary_integrity {
            match binary.verify_full() {
                Ok(_) => result.binary_ok = true,
                Err(e) => {
                    result.binary_ok = false;
                    result.errors.push(format!("binary: {}", e));
                }
            }
        }

        if let Some(ref library) = self.library_integrity {
            match library.verify_all() {
                Ok(mismatches) => {
                    result.library_ok = mismatches.is_empty();
                    for lib in &mismatches {
                        result.errors.push(format!("library mismatch: {}", lib.name));
                    }
                }
                Err(e) => {
                    result.library_ok = false;
                    result.errors.push(format!("library: {}", e));
                }
            }
        }

        if let Some(ref memory) = self.memory_integrity {
            match memory.verify_all() {
                Ok(modified) => {
                    result.memory_ok = modified.is_empty();
                    for idx in modified {
                        result.errors.push(format!("memory region {} modified", idx));
                    }
                }
                Err(e) => {
                    result.memory_ok = false;
                    result.errors.push(format!("memory: {}", e));
                }
            }
        }

        let debugger = PlatformDebugger::new();
        match debugger.is_debugger_present() {
            Ok(present) => result.debugger_detected = present,
            Err(e) => {
                result.errors.push(format!("debugger check: {}", e));
            }
        }

        Ok(result)
    }

    pub fn on_event(&mut self, callback: EventCallback) {
        self.dispatcher.register(callback);
    }

    fn startup_verification(&self) -> Result<()> {
        self.dispatcher.dispatch(Event::VerificationStarted);

        let debugger = PlatformDebugger::new();
        match debugger.is_debugger_present() {
            Ok(true) => {
                self.dispatcher.dispatch(Event::DebuggerDetected);
                if let Some(ref engine) = self.policy_engine {
                    let action = engine.evaluate(&Event::DebuggerDetected);
                    self.dispatcher.dispatch(Event::PolicyAction {
                        event: "DebuggerDetected".into(),
                        action: format!("{:?}", action),
                    });
                    if engine.execute(&action, &Event::DebuggerDetected) {
                        std::process::exit(1);
                    }
                }
            }
            Ok(false) => {}
            Err(e) => {
                self.dispatcher.dispatch(Event::Error {
                    message: format!("debugger check failed: {}", e),
                });
            }
        }

        self.dispatcher.dispatch(Event::VerificationCompleted);
        Ok(())
    }
}

impl Drop for RuntimeShield {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub binary_ok: bool,
    pub library_ok: bool,
    pub memory_ok: bool,
    pub debugger_detected: bool,
    pub errors: Vec<String>,
}

impl VerificationResult {
    pub fn new() -> Self {
        Self {
            binary_ok: true,
            library_ok: true,
            memory_ok: true,
            debugger_detected: false,
            errors: Vec::new(),
        }
    }

    pub fn is_integrity_ok(&self) -> bool {
        self.binary_ok && self.library_ok && self.memory_ok
    }
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self::new()
    }
}
