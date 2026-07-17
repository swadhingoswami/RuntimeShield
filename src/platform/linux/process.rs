use crate::core::error::{Error, Result};
use crate::platform::ProcessIdentity;

pub struct LinuxProcess;

impl LinuxProcess {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessIdentity for LinuxProcess {
    fn process_id(&self) -> u32 {
        std::process::id()
    }

    fn parent_process_id(&self) -> u32 {
        let content = std::fs::read_to_string("/proc/self/status").ok();
        if let Some(text) = content {
            for line in text.lines() {
                if line.starts_with("PPid:") {
                    if let Ok(ppid) = line.trim_start_matches("PPid:").trim().parse::<u32>() {
                        return ppid;
                    }
                }
            }
        }
        0
    }

    fn process_name(&self) -> Result<String> {
        let content = std::fs::read_to_string("/proc/self/status")
            .map_err(|e| Error::Platform(format!("failed to read /proc/self/status: {}", e)))?;

        for line in content.lines() {
            if line.starts_with("Name:") {
                return Ok(line.trim_start_matches("Name:").trim().to_string());
            }
        }

        Err(Error::Platform("could not find process name".into()))
    }

    fn executable_path(&self) -> Result<String> {
        let path = std::fs::read_link("/proc/self/exe")
            .map_err(|e| Error::Platform(format!("failed to read /proc/self/exe: {}", e)))?;
        Ok(path.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_id() {
        let proc = LinuxProcess::new();
        assert!(proc.process_id() > 0);
    }

    #[test]
    fn test_process_name() {
        let proc = LinuxProcess::new();
        match proc.process_name() {
            Ok(name) => assert!(!name.is_empty()),
            Err(e) => assert!(
                e.to_string().contains("/proc"),
                "Unexpected error: {}",
                e
            ),
        }
    }
}
