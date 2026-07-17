use crate::config::policy::PolicyConfig;
use crate::events::Event;
use crate::policy::Action;
use log::{error, warn};

pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(&self, event: &Event) -> Action {
        let action_str = match event {
            Event::DebuggerDetected => self.config.debugger_detected.as_deref(),
            Event::BinaryModified => self.config.binary_modified.as_deref(),
            Event::LibraryModified => self.config.library_modified.as_deref(),
            Event::HashMismatch { .. } => self.config.hash_mismatch.as_deref(),
            Event::MemoryIntegrityFailed => self.config.memory_modified.as_deref(),
            Event::VerificationStarted | Event::VerificationCompleted => None,
            Event::PolicyAction { .. } => None,
            Event::Error { .. } => Some("Log"),
            Event::Info { .. } => Some("Ignore"),
        };

        let action = action_str
            .map(|s| s.parse::<Action>().unwrap_or_else(|_| {
                log::warn!("invalid policy action '{}', falling back to Log", s);
                Action::Log
            }))
            .unwrap_or(Action::Ignore);

        match &action {
            Action::Log => warn!("Policy action: Log for event: {:?}", event),
            Action::Terminate => {
                error!("Policy action: Terminate for event: {:?}", event);
            }
            _ => {}
        }

        action
    }

    /// Execute the policy action. Returns true if the program should terminate.
    /// Tests should check this instead of relying on process exit.
    pub fn execute(&self, action: &Action, event: &Event) -> bool {
        match action {
            Action::Log => {
                warn!("Policy action: Log for event: {:?}", event);
                false
            }
            Action::Terminate => {
                error!("Policy action: Terminate for event: {:?}", event);
                true
            }
            Action::Callback => {
                log::info!("Policy action: Callback for event: {:?}", event);
                false
            }
            Action::Ignore => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminate_on_debugger() {
        let config = PolicyConfig {
            debugger_detected: Some("Terminate".into()),
            ..Default::default()
        };
        let engine = PolicyEngine::new(config);
        let action = engine.evaluate(&Event::DebuggerDetected);
        assert_eq!(action, Action::Terminate);
        assert!(engine.execute(&action, &Event::DebuggerDetected));
    }

    #[test]
    fn test_log_on_hash_mismatch() {
        let config = PolicyConfig {
            hash_mismatch: Some("Log".into()),
            ..Default::default()
        };
        let engine = PolicyEngine::new(config);
        let action = engine.evaluate(&Event::HashMismatch {
            expected: "abc".into(),
            actual: "def".into(),
        });
        assert_eq!(action, Action::Log);
    }

    #[test]
    fn test_ignore() {
        let config = PolicyConfig {
            binary_modified: Some("Ignore".into()),
            ..Default::default()
        };
        let engine = PolicyEngine::new(config);
        let action = engine.evaluate(&Event::BinaryModified);
        assert_eq!(action, Action::Ignore);
    }
}
