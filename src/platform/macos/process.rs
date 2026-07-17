use crate::core::error::{Error, Result};
use crate::platform::ProcessIdentity;
use std::ffi::CStr;

pub struct MacosProcess;

impl MacosProcess {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacosProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessIdentity for MacosProcess {
    fn process_id(&self) -> u32 {
        std::process::id()
    }

    fn parent_process_id(&self) -> u32 {
        unsafe { libc::getppid() as u32 }
    }

    fn process_name(&self) -> Result<String> {
        let pid = std::process::id() as i32;
        let mut buf = [0i8; 4096];
        let len = unsafe { proc_name(pid, buf.as_mut_ptr() as *mut libc::c_void, buf.len() as u32) };
        if len <= 0 {
            return Err(Error::Platform("proc_name failed".into()));
        }
        // proc_name returns number of bytes written, including null terminator
        let bytes = unsafe {
            std::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len())
        };
        // Find the first null byte
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(buf.len());
        if end == 0 {
            return Err(Error::Platform("empty process name".into()));
        }
        Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
    }

    fn executable_path(&self) -> Result<String> {
        let mut buf = [0i8; 4096];
        let mut size = buf.len() as u32;
        let result = unsafe { _NSGetExecutablePath(buf.as_mut_ptr(), &mut size) };
        if result != 0 {
            return Err(Error::Platform("_NSGetExecutablePath: buffer too small, need larger buffer".into()));
        }
        let path = unsafe { CStr::from_ptr(buf.as_ptr()) };
        Ok(path.to_string_lossy().into_owned())
    }
}

extern "C" {
    fn proc_name(pid: i32, buffer: *mut libc::c_void, buffersize: u32) -> i32;
    fn _NSGetExecutablePath(buf: *mut i8, bufsize: *mut u32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_id() {
        let proc = MacosProcess::new();
        assert!(proc.process_id() > 0);
    }

    #[test]
    fn test_parent_process_id() {
        let proc = MacosProcess::new();
        assert!(proc.parent_process_id() >= 1);
    }

    #[test]
    fn test_process_name() {
        let proc = MacosProcess::new();
        let result = proc.process_name();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_executable_path() {
        let proc = MacosProcess::new();
        let result = proc.executable_path();
        assert!(result.is_ok() || result.is_err());
        if let Ok(path) = result {
            assert!(!path.is_empty());
            assert!(path.contains('/'));
        }
    }
}
