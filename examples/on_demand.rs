use runtimeshield::RuntimeShield;
use std::sync::Arc;
use std::time::Duration;

/// Example demonstrating on-demand verification.
///
/// Run with: cargo run --example on_demand
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        .enable_binary_integrity()
        .enable_anti_debug()
        .on_event(Arc::new(|event| {
            println!("  Event: {:?}", event);
        }))
        .build()?;

    shield.start()?;
    println!("RuntimeShield initialized. Performing periodic on-demand checks...\n");

    for i in 0..5 {
        println!("--- Check {} ---", i + 1);

        let result = shield.verify_now()?;

        println!(
            "  Integrity: {}",
            if result.is_integrity_ok() {
                "PASS"
            } else {
                "FAIL"
            }
        );
        println!(
            "  Binary: {}",
            if result.binary_ok { "OK" } else { "MODIFIED" }
        );
        println!(
            "  Debugger: {}",
            if result.debugger_detected {
                "DETECTED"
            } else {
                "Not detected"
            }
        );

        if !result.errors.is_empty() {
            for err in &result.errors {
                println!("  Error: {}", err);
            }
        }

        println!();
        std::thread::sleep(Duration::from_secs(2));
    }

    shield.stop();
    println!("RuntimeShield stopped.");
    Ok(())
}
