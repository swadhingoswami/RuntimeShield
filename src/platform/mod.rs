pub mod linux;
pub mod macos;
pub mod windows;

use crate::core::error::Result;

pub trait ProcessIdentity {
    fn process_id(&self) -> u32;
    fn parent_process_id(&self) -> u32;
    fn process_name(&self) -> Result<String>;
    fn executable_path(&self) -> Result<String>;
}

pub trait DebuggerDetector {
    fn is_debugger_present(&self) -> Result<bool>;
    fn debugger_info(&self) -> Result<String>;
}

pub trait MemoryRegionReader {
    fn read_region(&self, address: usize, size: usize) -> Result<Vec<u8>>;
    fn get_code_regions(&self) -> Result<Vec<(usize, usize)>>;
}

#[cfg(target_os = "linux")]
pub type PlatformDebugger = linux::debugger::LinuxDebugger;
#[cfg(target_os = "linux")]
pub type PlatformProcess = linux::process::LinuxProcess;

#[cfg(target_os = "macos")]
pub type PlatformDebugger = macos::debugger::MacosDebugger;
#[cfg(target_os = "macos")]
pub type PlatformProcess = macos::process::MacosProcess;

#[cfg(target_os = "linux")]
pub type PlatformMemory = linux::memory::LinuxMemory;

#[cfg(target_os = "macos")]
pub type PlatformMemory = macos::memory::MacosMemory;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub type PlatformDebugger = windows::WindowsDebugger;
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub type PlatformProcess = windows::WindowsProcess;
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub type PlatformMemory = windows::WindowsMemory;
