# API Guide

## Installation

Add RuntimeShield to your `Cargo.toml`:

```toml
[dependencies]
runtimeshield = "0.1"
```

## Quick Start

```rust
use std::sync::Arc;
use runtimeshield::RuntimeShield;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        .enable_startup_verification()
        .enable_runtime_monitor()
        .enable_binary_integrity()
        .enable_library_integrity()
        .enable_memory_integrity()
        .enable_anti_debug()
        .monitor_interval(5000)
        .on_event(Arc::new(|event| {
            println!("{:?}", event);
        }))
        .build()?;

    shield.start()?;
    
    // Your application code here
    
    shield.stop();
    Ok(())
}
```

## Builder API

### `RuntimeShield::builder()`

Creates a new `RuntimeShieldBuilder`.

### Builder Methods

| Method | Default | Description |
|---|---|---|
| `enable_startup_verification()` | `false` | Verify at startup |
| `enable_runtime_monitor()` | `false` | Periodic background verification |
| `enable_binary_integrity()` | `false` | Check binary on disk |
| `enable_library_integrity()` | `false` | Check loaded libraries |
| `enable_process_identity()` | `false` | Verify process identity |
| `enable_memory_integrity()` | `false` | Check memory regions |
| `enable_anti_debug()` | `false` | Detect debuggers |
| `monitor_interval(ms)` | `5000` | Interval for runtime monitor |
| `policy(path)` | `None` | Path to policy TOML file |
| `manifest(path)` | `None` | Path to manifest JSON file |
| `on_event(callback)` | `None` | Register event callback |

### `build() -> Result<RuntimeShield>`

Constructs the `RuntimeShield` instance. May fail if:
- Policy file is specified but cannot be read or parsed
- Configuration is invalid

## Instance Methods

### `start() -> Result<()>`

Starts RuntimeShield protection:
1. Runs startup verification (if enabled)
2. Initializes integrity modules
3. Starts runtime monitor (if enabled)

### `stop()`

Stops RuntimeShield:
1. Stops the runtime monitor thread
2. Cleans up resources

### `verify_now() -> Result<VerificationResult>`

Performs on-demand verification of all enabled modules. Returns current integrity status.

### `on_event(callback)`

Registers an additional event callback. Can be called multiple times.

## VerificationResult

```rust
pub struct VerificationResult {
    pub binary_ok: bool,
    pub library_ok: bool,
    pub memory_ok: bool,
    pub debugger_detected: bool,
    pub errors: Vec<String>,
}
```

### Methods

| Method | Returns | Description |
|---|---|---|
| `is_integrity_ok()` | `bool` | All integrity checks passed |

## Event Callback

```rust
pub type EventCallback = Arc<dyn Fn(Event) + Send + Sync>;
```

## Policy File

```toml
# runtime_policy.toml
DebuggerDetected = "Terminate"
BinaryModified = "Terminate"
LibraryModified = "Callback"
HashMismatch = "Log"
MemoryModified = "Log"
```

## Error Handling

All fallible methods return `Result<T, Error>`:

```rust
pub enum Error {
    Config(String),
    Io(std::io::Error),
    HashMismatch { expected: String, actual: String },
    Manifest(String),
    Platform(String),
    Policy(String),
    Verification(String),
    AlreadyInitialized,
    NotInitialized,
    Unknown(String),
}
```

## Full Example

```rust
use std::sync::Arc;
use runtimeshield::{RuntimeShield, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut shield = RuntimeShield::builder()
        .enable_startup_verification()
        .enable_runtime_monitor()
        .enable_binary_integrity()
        .enable_library_integrity()
        .enable_anti_debug()
        .monitor_interval(10000)
        .policy("runtime_policy.toml")
        .on_event(Arc::new(|event: Event| {
            match event {
                Event::DebuggerDetected => {
                    eprintln!("SECURITY: Debugger detected!");
                }
                Event::BinaryModified => {
                    eprintln!("SECURITY: Binary modified!");
                }
                Event::LibraryModified => {
                    eprintln!("SECURITY: Library modified!");
                }
                Event::VerificationCompleted => {
                    // Normal operation
                }
                Event::PolicyAction { event, action } => {
                    eprintln!("POLICY: {} → {}", event, action);
                }
                _ => {}
            }
        }))
        .build()?;

    shield.start()?;
    println!("RuntimeShield protection active");

    // On-demand check
    let result = shield.verify_now()?;
    if !result.is_integrity_ok() {
        eprintln!("Integrity issues found: {:?}", result.errors);
    }

    // Keep running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```
