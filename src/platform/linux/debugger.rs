use crate::core::error::{Error, Result};
use crate::platform::DebuggerDetector;

pub struct LinuxDebugger;

impl LinuxDebugger {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxDebugger {
    fn default() -> Self {
        Self::new()
    }
}

impl DebuggerDetector for LinuxDebugger {
    fn is_debugger_present(&self) -> Result<bool> {
        let tracer_pid = self.read_tracer_pid()?;
        Ok(tracer_pid != 0)
    }

    fn debugger_info(&self) -> Result<String> {
        let tracer_pid = self.read_tracer_pid()?;
        Ok(format!("TracerPid={}", tracer_pid))
    }
}

impl LinuxDebugger {
    fn read_tracer_pid(&self) -> Result<u32> {
        let content = std::fs::read_to_string("/proc/self/status")
            .map_err(|e| Error::Platform(format!("failed to read /proc/self/status: {}", e)))?;

        for line in content.lines() {
            if line.starts_with("TracerPid:") {
                let pid_str = line.trim_start_matches("TracerPid:").trim();
                return pid_str
                    .parse::<u32>()
                    .map_err(|e| Error::Platform(format!("invalid TracerPid: {}", e)));
            }
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_not_present() {
        let detector = LinuxDebugger::new();
        match detector.is_debugger_present() {
            Ok(present) => {
                // In normal test runs, no debugger should be attached
                assert!(!present);
            }
            Err(e) => {
                // On systems without /proc, this is acceptable
                assert!(e.to_string().contains("/proc"), "Unexpected error: {}", e);
            }
        }
    }
}
