use runtimeshield::crypto::hash::hash_bytes;
use runtimeshield::crypto::merkle;
use runtimeshield::integrity::binary::BinaryIntegrity;
use runtimeshield::integrity::library::LibraryIntegrity;
use runtimeshield::integrity::memory::MemoryIntegrity;
use runtimeshield::events::Event;
use runtimeshield::policy::Action;
use runtimeshield::config::policy::PolicyConfig;
use runtimeshield::policy::engine::PolicyEngine;

#[test]
fn test_full_integrity_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let bin_path = dir.path().join("test_app");
    let data = vec![0x42u8; 65536]; // 64KB binary
    std::fs::write(&bin_path, &data).unwrap();

    let mut integrity = BinaryIntegrity::new(bin_path.to_string_lossy().to_string());
    let manifest = integrity.generate_manifest("1.0.0").unwrap();

    assert_eq!(manifest.total_pages, 16); // 64KB / 4KB = 16
    assert!(!manifest.root_hash.is_empty());

    integrity.load_manifest(manifest.clone());
    assert!(integrity.verify_full().is_ok());

    // Verify each page
    for i in 0..16 {
        assert!(integrity.verify_page(i).unwrap());
    }
}

#[test]
fn test_detects_binary_modification() {
    let dir = tempfile::tempdir().unwrap();
    let bin_path = dir.path().join("test_app");
    std::fs::write(&bin_path, vec![0x42u8; 65536]).unwrap();

    let mut integrity = BinaryIntegrity::new(bin_path.to_string_lossy().to_string());
    let manifest = integrity.generate_manifest("1.0.0").unwrap();
    integrity.load_manifest(manifest);

    // Modify the binary
    std::fs::write(&bin_path, vec![0xFFu8; 65536]).unwrap();

    assert!(integrity.verify_full().is_err());
}

#[test]
fn test_merkle_tree_consistency() {
    let data = b"The quick brown fox jumps over the lazy dog";
    let tree = merkle::build_merkle_tree(data);
    assert_eq!(tree.leaf_count, 1);
    assert_eq!(tree.levels, 1);

    // Verify page hash
    assert!(merkle::verify_page_hash(&tree, data, 0));
}

#[test]
fn test_policy_evaluation() {
    let config = PolicyConfig {
        debugger_detected: Some("Terminate".into()),
        binary_modified: Some("Callback".into()),
        library_modified: Some("Log".into()),
        hash_mismatch: Some("Ignore".into()),
        memory_modified: Some("Callback".into()),
    };

    let engine = PolicyEngine::new(config);

    assert_eq!(engine.evaluate(&Event::DebuggerDetected), Action::Terminate);
    assert_eq!(engine.evaluate(&Event::BinaryModified), Action::Callback);
    assert_eq!(engine.evaluate(&Event::LibraryModified), Action::Log);
    assert_eq!(engine.evaluate(&Event::MemoryIntegrityFailed), Action::Callback);
    assert_eq!(
        engine.evaluate(&Event::HashMismatch {
            expected: "abc".into(),
            actual: "def".into()
        }),
        Action::Ignore
    );
}

#[test]
fn test_library_integrity_empty() {
    let li = LibraryIntegrity::new();
    assert!(li.loaded_libraries().is_empty());

    let mismatches = li.verify_all().unwrap();
    assert!(mismatches.is_empty());
}

#[test]
fn test_memory_integrity_regions() {
    let mut mi = MemoryIntegrity::new();
    assert!(mi.protected_regions().is_empty());

    mi.add_protected_region(0x1000, 4096);
    mi.add_protected_region(0x2000, 4096);
    assert_eq!(mi.protected_regions().len(), 2);
}

#[test]
fn test_hash_comparison() {
    let data1 = b"identical data";
    let data2 = b"identical data";
    let data3 = b"different data";

    assert_eq!(hash_bytes(data1), hash_bytes(data2));
    assert_ne!(hash_bytes(data1), hash_bytes(data3));
}

#[test]
fn test_manifest_generation_and_loading() {
    let dir = tempfile::tempdir().unwrap();
    let bin_path = dir.path().join("test_binary");

    // Use a data size that produces multiple pages
    let data = vec![0xABu8; merkle::PAGE_SIZE * 5 + 1234];
    std::fs::write(&bin_path, &data).unwrap();

    let integrity = BinaryIntegrity::new(bin_path.to_string_lossy().to_string());
    let manifest = integrity.generate_manifest("2.0.0").unwrap();

    assert_eq!(manifest.total_pages, 6);
    assert_eq!(manifest.file_size, data.len() as u64);
    assert_eq!(manifest.version, "2.0.0");

    // Save and reload
    let json = serde_json::to_string(&manifest).unwrap();
    let manifest_path = dir.path().join("test.manifest.json");
    std::fs::write(&manifest_path, &json).unwrap();

    let mut loaded = BinaryIntegrity::new(bin_path.to_string_lossy().to_string());
    loaded.load_manifest_from_path(&manifest_path).unwrap();
    assert!(loaded.verify_full().is_ok());
}

#[test]
fn test_default_policy() {
    let config = PolicyConfig::default();
    assert_eq!(config.debugger_detected, Some("Terminate".into()));
    assert_eq!(config.binary_modified, Some("Terminate".into()));
    assert_eq!(config.library_modified, Some("Log".into()));
}

#[test]
fn test_policy_action_execution() {
    let config = PolicyConfig::default();
    let engine = PolicyEngine::new(config);

    assert!(engine.execute(&Action::Terminate, &Event::DebuggerDetected));
    assert!(!engine.execute(&Action::Log, &Event::LibraryModified));
    assert!(!engine.execute(&Action::Callback, &Event::BinaryModified));
    assert!(!engine.execute(&Action::Ignore, &Event::DebuggerDetected));
}
