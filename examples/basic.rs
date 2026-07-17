use runtimeshield::{RuntimeShield, Event};
use std::sync::Arc;
use std::time::Duration;

/// Basic example demonstrating RuntimeShield with all protections enabled.
///
/// Run with: cargo run --example basic
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        // Startup verification
        .enable_startup_verification()
        // Periodic background monitoring
        .enable_runtime_monitor()
        // Verify binary integrity on disk
        .enable_binary_integrity()
        // Verify loaded shared libraries
        .enable_library_integrity()
        // Verify executable memory regions
        .enable_memory_integrity()
        // Detect debugger attachment
        .enable_anti_debug()
        // Check every 5 seconds
        .monitor_interval(5000)
        // Register event handler
        .on_event(Arc::new(|event: Event| {
            match event {
                Event::DebuggerDetected => {
                    eprintln!("  [WARNING] Debugger detected!");
                }
                Event::BinaryModified => {
                    eprintln!("  [CRITICAL] Binary has been modified!");
                }
                Event::LibraryModified => {
                    eprintln!("  [WARNING] A library has been modified!");
                }
                Event::MemoryIntegrityFailed => {
                    eprintln!("  [CRITICAL] Memory integrity violation!");
                }
                Event::VerificationStarted => {
                    println!("  [CHECK] Starting verification cycle...");
                }
                Event::VerificationCompleted => {
                    println!("  [CHECK] Verification cycle complete.");
                }
                Event::PolicyAction { event, action } => {
                    println!("  [POLICY] {} → {}", event, action);
                }
                Event::Error { message } => {
                    eprintln!("  [ERROR] {}", message);
                }
                Event::Info { message } => {
                    println!("  [INFO] {}", message);
                }
                Event::HashMismatch { expected, actual } => {
                    eprintln!("  [WARNING] Hash mismatch: expected={}, actual={}", expected, actual);
                }
            }
        }))
        .build()?;

    println!("Starting RuntimeShield...");
    shield.start()?;
    println!("RuntimeShield is active. Monitoring every 5 seconds.");

    // Perform on-demand verification
    println!("\nPerforming on-demand verification...");
    let result = shield.verify_now()?;
    println!("  Binary integrity: {}", if result.binary_ok { "OK" } else { "FAILED" });
    println!("  Library integrity: {}", if result.library_ok { "OK" } else { "FAILED" });
    println!("  Memory integrity: {}", if result.memory_ok { "OK" } else { "FAILED" });
    println!("  Debugger detected: {}", result.debugger_detected);

    if !result.is_integrity_ok() {
        eprintln!("\n  Integrity check FAILED:");
        for err in &result.errors {
            eprintln!("    - {}", err);
        }
    } else {
        println!("\n  All integrity checks passed.");
    }

    // Keep running for a while to demonstrate runtime monitoring
    println!("\nRuntime monitoring active. Press Ctrl+C to stop...");
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
