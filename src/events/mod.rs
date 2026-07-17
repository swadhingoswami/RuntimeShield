pub mod callback;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    DebuggerDetected,
    BinaryModified,
    LibraryModified,
    HashMismatch { expected: String, actual: String },
    MemoryIntegrityFailed,
    VerificationStarted,
    VerificationCompleted,
    PolicyAction { event: String, action: String },
    Error { message: String },
    Info { message: String },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::DebuggerDetected => write!(f, "DebuggerDetected"),
            Event::BinaryModified => write!(f, "BinaryModified"),
            Event::LibraryModified => write!(f, "LibraryModified"),
            Event::HashMismatch { expected, actual } => {
                write!(f, "HashMismatch(expected={}, actual={})", expected, actual)
            }
            Event::MemoryIntegrityFailed => write!(f, "MemoryIntegrityFailed"),
            Event::VerificationStarted => write!(f, "VerificationStarted"),
            Event::VerificationCompleted => write!(f, "VerificationCompleted"),
            Event::PolicyAction { event, action } => {
                write!(f, "PolicyAction(event={}, action={})", event, action)
            }
            Event::Error { message } => write!(f, "Error({})", message),
            Event::Info { message } => write!(f, "Info({})", message),
        }
    }
}
