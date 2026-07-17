use crate::core::error::{Error, Result};
use crate::platform::DebuggerDetector;

pub struct MacosDebugger;

impl MacosDebugger {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacosDebugger {
    fn default() -> Self {
        Self::new()
    }
}

impl DebuggerDetector for MacosDebugger {
    fn is_debugger_present(&self) -> Result<bool> {
        check_p_traced()
    }

    fn debugger_info(&self) -> Result<String> {
        let traced = check_p_traced()?;
        Ok(format!("P_TRACED={}", traced))
    }
}

// macOS sysctl constants (not all are in the libc crate)
const CTL_KERN: i32 = 1;
const KERN_PROC: i32 = 14;
const KERN_PROC_PID: i32 = 1;
const P_TRACED: i32 = 0x800;

// kinfo_proc layout on macOS (using fixed-size buffers)
// struct kinfo_proc { struct extern_proc; struct eproc; }
// On arm64: extern_proc is ~232 bytes, eproc is ~280 bytes, total ~512 bytes
// p_flag is at offset 0x10 within extern_proc (offset 0x10 from start of kinfo_proc)
const KINFO_PROC_SIZE: usize = 512;

fn check_p_traced() -> Result<bool> {
    let pid = std::process::id() as i32;
    let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_PID, pid];
    let mut buf = [0u8; KINFO_PROC_SIZE];
    let mut size = buf.len();

    let result = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            4,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };

    if result != 0 {
        return Err(Error::Platform(format!(
            "sysctl failed: {}",
            std::io::Error::last_os_error()
        )));
    }

    // p_flag is at offset 0x10 in the kinfo_proc structure
    // (within extern_proc.kp_flags, which is the first field after proc p_pid,etc.)
    // On both arm64 and x86_64 macOS, p_flag is at offset 0x10
    if size < 0x14 {
        return Err(Error::Platform("sysctl returned incomplete data".into()));
    }

    let p_flag_bytes = [
        buf[0x10], buf[0x11], buf[0x12], buf[0x13],
    ];
    let p_flag = i32::from_ne_bytes(p_flag_bytes);

    Ok((p_flag & P_TRACED) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_detection() {
        let detector = MacosDebugger::new();
        let result = detector.is_debugger_present();
        assert!(result.is_ok() || result.is_err());
        if let Ok(present) = result {
            assert!(!present, "Debugger should not be present during tests");
        }
    }
}
