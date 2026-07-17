# Process Identity

## Overview

Process identity verification helps an application confirm that it is running in the expected context. This includes verifying process ID, parent process, process name, and executable path.

## API

```rust
pub trait ProcessIdentity {
    fn process_id(&self) -> u32;
    fn parent_process_id(&self) -> u32;
    fn process_name(&self) -> Result<String>;
    fn executable_path(&self) -> Result<String>;
}
```

## Linux Implementation

On Linux, process identity is obtained from the `/proc` filesystem:

```rust
struct LinuxProcess;

impl ProcessIdentity for LinuxProcess {
    fn process_id(&self) -> u32 {
        std::process::id()
    }

    fn parent_process_id(&self) -> u32 {
        // Read PPid from /proc/self/status
    }

    fn process_name(&self) -> Result<String> {
        // Read Name from /proc/self/status
    }

    fn executable_path(&self) -> Result<String> {
        // Read /proc/self/exe symlink
    }
}
```

## Use Cases

### Detecting Forked Processes

If an application expects to be run directly (not forked from another process), the parent PID can be checked:

```rust
let proc = PlatformProcess::new();
let ppid = proc.parent_process_id();
if ppid != 1 && ppid != launch_daemon_pid {
    // Unexpected parent process
}
```

### Verifying Executable Path

The executable path should match the expected installation directory:

```rust
let proc = PlatformProcess::new();
let exe_path = proc.executable_path()?;
if !exe_path.starts_with("/opt/myapp/") {
    // Application was moved or replaced
}
```

### Confirming Process Name

```rust
let proc = PlatformProcess::new();
let name = proc.process_name()?;
if name != "myapp" {
    // Binary was renamed or is running under a different name
}
```

## Trust Assumptions

Process identity information from platform APIs is generally reliable, but with important caveats:

| Source | Trust Level | Notes |
|---|---|---|
| `/proc/self/status` | Medium | Can be manipulated by kernel modules or root |
| `/proc/self/exe` | Medium | Symlink can be manipulated |
| `getppid()` | High | Direct syscall, harder to intercept |
| macOS `proc_name` | Medium | Depends on Mach API integrity |

## Limitations

1. **Process identity data can be spoofed** by root or kernel-mode code.

2. **Process name may differ from executable name** — Applications can rename their process (e.g., `prctl(PR_SET_NAME)` on Linux).

3. **Executable path may be misleading** — Symbolic links and bind mounts can make the path differ from the actual file.

4. **Parent process ID can change** — If the parent process exits, the orphaned process is adopted by init (PID 1).

## Windows Design

The `ProcessIdentity` trait can be implemented for Windows using:

- `GetCurrentProcessId()` — PID
- `GetModuleFileName(NULL)` — Executable path
- `GetProcessImageFileName` — Process name
- `NtQueryInformationProcess` — Parent PID

Implementation is deferred pending platform support.
