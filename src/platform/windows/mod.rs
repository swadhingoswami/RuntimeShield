// Windows platform support is designed but implementation is deferred.
// The trait-based abstraction allows adding Windows implementation
// without modifying higher-level modules.
//
// To implement Windows support:
// 1. Implement ProcessIdentity for WindowsProcess
// 2. Implement DebuggerDetector for WindowsDebugger (using IsDebuggerPresent, NtQueryInformationProcess, etc.)
// 3. Implement MemoryRegionReader for WindowsMemory (using VirtualQuery, ReadProcessMemory, etc.)
//
// Windows-specific crates needed:
// - windows-sys or winapi

pub struct WindowsProcess;
pub struct WindowsDebugger;
pub struct WindowsMemory;

impl WindowsProcess {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsDebugger {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsDebugger {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsMemory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsMemory {
    fn default() -> Self {
        Self::new()
    }
}
