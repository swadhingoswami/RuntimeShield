use runtimeshield::platform::DebuggerDetector;
use runtimeshield::platform::PlatformDebugger;
use runtimeshield::platform::PlatformProcess;
use runtimeshield::platform::ProcessIdentity;

#[test]
fn test_platform_process_id() {
    let proc = PlatformProcess::new();
    let pid = proc.process_id();
    assert!(pid > 0, "Process ID should be positive, got {}", pid);
}

#[test]
fn test_platform_debugger_detection() {
    let detector = PlatformDebugger::new();
    // In a normal test run, no debugger should be attached
    let present = detector.is_debugger_present();
    match present {
        Ok(p) => assert!(!p, "Debugger should not be present during tests"),
        Err(_) => {
            // Platform-specific debugger detection may not be available in all environments
        }
    }
}

#[test]
fn test_platform_executable_path() {
    let proc = PlatformProcess::new();
    match proc.executable_path() {
        Ok(path) => {
            assert!(!path.is_empty(), "Executable path should not be empty");
            assert!(
                path.contains('/') || path.contains('\\'),
                "Executable path should be absolute"
            );
        }
        Err(e) => {
            let msg = e.to_string();
            // On Linux, errors come from /proc; on macOS, errors come from various system calls
            assert!(
                msg.contains("/proc") || msg.contains("too small") || msg.contains("failed"),
                "Unexpected error: {}",
                msg
            );
        }
    }
}

#[test]
fn test_platform_process_name() {
    let proc = PlatformProcess::new();
    match proc.process_name() {
        Ok(name) => {
            assert!(!name.is_empty(), "Process name should not be empty");
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("/proc") || msg.contains("failed") || msg.contains("empty"),
                "Unexpected error: {}",
                msg
            );
        }
    }
}

#[test]
fn test_platform_parent_pid() {
    let proc = PlatformProcess::new();
    let ppid = proc.parent_process_id();
    // On macOS, ppid should be >= 1
    if cfg!(target_os = "macos") {
        assert!(ppid >= 1, "Parent PID should be >= 1, got {}", ppid);
    }
    // On Linux, test init/systemd parent
}
