use runtimeshield::integrity::binary::BinaryIntegrity;

/// Example demonstrating manifest generation.
///
/// Run with: cargo run --example manifest_gen
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    println!("Generating manifest for: {}", exe_path.display());

    let integrity = BinaryIntegrity::new(exe_path.to_string_lossy().to_string());
    let manifest = integrity.generate_manifest("1.0.0")?;

    let json = serde_json::to_string_pretty(&manifest)?;
    let manifest_path = exe_path.with_extension("manifest.json");
    std::fs::write(&manifest_path, &json)?;

    println!("Manifest generated successfully:");
    println!("  Path: {}", manifest_path.display());
    println!("  Pages: {}", manifest.total_pages);
    println!("  File size: {} bytes", manifest.file_size);
    println!("  Root hash: {}", manifest.root_hash);
    println!("  Version: {}", manifest.version);
    println!("  Timestamp: {}", manifest.timestamp);

    // Verify the manifest
    let mut verifier = BinaryIntegrity::new(exe_path.to_string_lossy().to_string());
    verifier.load_manifest(manifest);
    match verifier.verify_full() {
        Ok(_) => println!("\nSelf-verification: PASSED"),
        Err(e) => println!("\nSelf-verification: FAILED - {}", e),
    }

    Ok(())
}
