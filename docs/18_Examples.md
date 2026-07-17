# Examples

## Basic Protection

Minimal setup with all protections enabled:

```rust
use runtimeshield::RuntimeShield;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        .enable_startup_verification()
        .enable_runtime_monitor()
        .enable_binary_integrity()
        .enable_library_integrity()
        .enable_memory_integrity()
        .enable_anti_debug()
        .build()?;

    shield.start()?;
    // ... application logic ...
    shield.stop();
    Ok(())
}
```

## With Custom Callback

Register a callback to handle events:

```rust
use runtimeshield::{RuntimeShield, Event};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        .enable_anti_debug()
        .enable_binary_integrity()
        .on_event(Arc::new(|event: Event| {
            match event {
                Event::DebuggerDetected => {
                    // Alert security team
                    // Send metrics
                    // Log to audit trail
                }
                Event::BinaryModified => {
                    // Initiate graceful shutdown
                    // Preserve forensic data
                }
                Event::VerificationStarted | Event::VerificationCompleted => {
                    // Update health check status
                }
                _ => {}
            }
        }))
        .build()?;

    shield.start()?;
    Ok(())
}
```

## With Policy File

Use a TOML policy file for response configuration:

```rust
use runtimeshield::RuntimeShield;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        .enable_binary_integrity()
        .enable_anti_debug()
        .policy("runtime_policy.toml")
        .build()?;

    shield.start()?;
    Ok(())
}
```

Policy file content:

```toml
DebuggerDetected = "Terminate"
BinaryModified = "Callback"
LibraryModified = "Log"
HashMismatch = "Log"
MemoryModified = "Callback"
```

## On-Demand Verification

Verify integrity before sensitive operations:

```rust
use runtimeshield::RuntimeShield;
use std::sync::Arc;

fn process_payment(shield: &RuntimeShield) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure environment is clean before processing payment
    let result = shield.verify_now()?;

    if !result.is_integrity_ok() {
        return Err("Integrity check failed".into());
    }

    // Process payment
    Ok(())
}
```

## Server Application

Integrate RuntimeShield into a server:

```rust
use runtimeshield::RuntimeShield;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        .enable_runtime_monitor()
        .enable_binary_integrity()
        .enable_library_integrity()
        .enable_anti_debug()
        .monitor_interval(30000)
        .on_event(Arc::new(|event| {
            log::info!("RuntimeShield event: {:?}", event);
        }))
        .build()?;

    shield.start()?;

    // Start your server
    println!("Server starting with RuntimeShield protection...");

    // Block main thread
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

## Game Application

Integrate RuntimeShield into a game loop:

```rust
use runtimeshield::RuntimeShield;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        .enable_startup_verification()
        .enable_binary_integrity()
        .enable_anti_debug()
        .build()?;

    shield.start()?;

    // Game loop
    loop {
        // Check integrity before each frame
        if let Ok(result) = shield.verify_now() {
            if !result.is_integrity_ok() {
                eprintln!("Game integrity compromised!");
                break;
            }
        }

        // Update game logic
        // Render frame
    }

    shield.stop();
    Ok(())
}
```

## Manifest Generation

Generate a manifest for your binary:

```rust
use runtimeshield::crypto::merkle::Manifest;
use runtimeshield::integrity::binary::BinaryIntegrity;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let integrity = BinaryIntegrity::new(exe_path.to_string_lossy().to_string());
    let manifest = integrity.generate_manifest(env!("CARGO_PKG_VERSION"))?;

    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write("app.manifest.json", &json)?;

    println!("Manifest generated: {} pages", manifest.total_pages);
    println!("Root hash: {}", manifest.root_hash);
    Ok(())
}
```
