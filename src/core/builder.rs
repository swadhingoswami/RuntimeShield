use crate::config::policy::PolicyConfig;
use crate::core::error::{Error, Result};
use crate::events::callback::EventCallback;
use crate::RuntimeShield;

#[derive(Clone)]
pub struct RuntimeShieldBuilder {
    pub(crate) startup_verification: bool,
    pub(crate) runtime_monitor: bool,
    pub(crate) binary_integrity: bool,
    pub(crate) library_integrity: bool,
    pub(crate) process_identity: bool,
    pub(crate) memory_integrity: bool,
    pub(crate) anti_debug: bool,
    pub(crate) policy_path: Option<String>,
    pub(crate) policy: Option<PolicyConfig>,
    pub(crate) callback: Option<EventCallback>,
    pub(crate) monitor_interval_ms: u64,
    pub(crate) manifest_path: Option<String>,
}

impl Default for RuntimeShieldBuilder {
    fn default() -> Self {
        Self {
            startup_verification: false,
            runtime_monitor: false,
            binary_integrity: false,
            library_integrity: false,
            process_identity: false,
            memory_integrity: false,
            anti_debug: false,
            policy_path: None,
            policy: None,
            callback: None,
            monitor_interval_ms: 5000,
            manifest_path: None,
        }
    }
}

impl RuntimeShieldBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable_startup_verification(mut self) -> Self {
        self.startup_verification = true;
        self
    }

    pub fn enable_runtime_monitor(mut self) -> Self {
        self.runtime_monitor = true;
        self
    }

    pub fn enable_binary_integrity(mut self) -> Self {
        self.binary_integrity = true;
        self
    }

    pub fn enable_library_integrity(mut self) -> Self {
        self.library_integrity = true;
        self
    }

    pub fn enable_process_identity(mut self) -> Self {
        self.process_identity = true;
        self
    }

    pub fn enable_memory_integrity(mut self) -> Self {
        self.memory_integrity = true;
        self
    }

    pub fn enable_anti_debug(mut self) -> Self {
        self.anti_debug = true;
        self
    }

    pub fn policy(mut self, path: impl Into<String>) -> Self {
        self.policy_path = Some(path.into());
        self
    }

    pub fn manifest(mut self, path: impl Into<String>) -> Self {
        self.manifest_path = Some(path.into());
        self
    }

    pub fn monitor_interval(mut self, millis: u64) -> Self {
        self.monitor_interval_ms = millis;
        self
    }

    pub fn on_event(mut self, callback: EventCallback) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn build(mut self) -> Result<RuntimeShield> {
        let policy = if let Some(ref path) = self.policy_path.clone() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| Error::Config(format!("failed to read policy file '{}': {}", path, e)))?;
            let cfg: PolicyConfig = toml::from_str(&content)
                .map_err(|e| Error::Config(format!("failed to parse policy file '{}': {}", path, e)))?;
            Some(cfg)
        } else {
            self.policy.take()
        };

        Ok(RuntimeShield::new(self, policy))
    }

    #[cfg(test)]
    pub fn with_policy_config(mut self, policy: PolicyConfig) -> Self {
        self.policy = Some(policy);
        self
    }
}
