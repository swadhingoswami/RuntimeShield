use crate::core::error::{Error, Result};
use crate::crypto::hash::{hash_bytes, hash_file, HashValue};
use crate::crypto::merkle::{build_manifest, build_merkle_tree, Manifest};
use std::io::{Read, Seek};

#[derive(Clone)]
pub struct BinaryIntegrity {
    executable_path: String,
    manifest: Option<Manifest>,
}

impl BinaryIntegrity {
    pub fn new(executable_path: impl Into<String>) -> Self {
        Self {
            executable_path: executable_path.into(),
            manifest: None,
        }
    }

    pub fn load_manifest(&mut self, manifest: Manifest) {
        self.manifest = Some(manifest);
    }

    pub fn load_manifest_from_path(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Manifest(format!("failed to read manifest: {}", e)))?;
        let manifest: Manifest = serde_json::from_str(&content)
            .map_err(|e| Error::Manifest(format!("failed to parse manifest: {}", e)))?;
        self.manifest = Some(manifest);
        Ok(())
    }

    pub fn generate_manifest(&self, version: &str) -> Result<Manifest> {
        let data = std::fs::read(&self.executable_path)
            .map_err(Error::Io)?;
        Ok(build_manifest(&data, version))
    }

    pub fn verify_full(&self) -> Result<()> {
        let manifest = self
            .manifest
            .as_ref()
            .ok_or_else(|| Error::Manifest("no manifest loaded".into()))?;

        let data = std::fs::read(&self.executable_path)
            .map_err(Error::Io)?;

        let tree = build_merkle_tree(&data);
        let computed_root = hex::encode(tree.root.hash);

        if computed_root != manifest.root_hash {
            return Err(Error::HashMismatch {
                expected: manifest.root_hash.clone(),
                actual: computed_root,
            });
        }

        Ok(())
    }

    pub fn verify_page(&self, page_index: usize) -> Result<bool> {
        let manifest = self
            .manifest
            .as_ref()
            .ok_or_else(|| Error::Manifest("no manifest loaded".into()))?;

        if page_index >= manifest.total_pages {
            return Err(Error::Verification(format!(
                "page index {} out of range (total: {})",
                page_index, manifest.total_pages
            )));
        }

        let entry = &manifest.entries[page_index];

        // Read only the page-sized chunk from the file, not the entire binary
        let mut file = std::fs::File::open(&self.executable_path)
            .map_err(Error::Io)?;

        file.seek(std::io::SeekFrom::Start(entry.offset)).map_err(Error::Io)?;

        let mut page_data = vec![0u8; entry.size];
        file.read_exact(&mut page_data).map_err(Error::Io)?;

        let actual_hash = hash_bytes(&page_data);
        Ok(hex::encode(actual_hash) == entry.page_hash)
    }

    pub fn verify_hash(&self) -> Result<HashValue> {
        hash_file(std::path::Path::new(&self.executable_path)).map_err(Error::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::merkle::PAGE_SIZE;

    #[test]
    fn test_generate_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_binary");
        std::fs::write(&path, vec![0xABu8; 10000]).unwrap();

        let integrity = BinaryIntegrity::new(path.to_string_lossy().to_string());
        let manifest = integrity.generate_manifest("1.0.0").unwrap();
        assert!(!manifest.root_hash.is_empty());
        assert!(manifest.total_pages > 0);
    }

    #[test]
    fn test_verify_full_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_binary");
        let data = vec![0x42u8; 5000];
        std::fs::write(&path, &data).unwrap();

        let mut integrity = BinaryIntegrity::new(path.to_string_lossy().to_string());
        let manifest = integrity.generate_manifest("1.0.0").unwrap();
        integrity.load_manifest(manifest);
        assert!(integrity.verify_full().is_ok());
    }

    #[test]
    fn test_verify_full_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_binary");
        std::fs::write(&path, vec![0x42u8; 5000]).unwrap();

        let mut integrity = BinaryIntegrity::new(path.to_string_lossy().to_string());
        let manifest = integrity.generate_manifest("1.0.0").unwrap();
        integrity.load_manifest(manifest);

        // Modify the binary
        std::fs::write(&path, vec![0xFFu8; 5000]).unwrap();

        assert!(integrity.verify_full().is_err());
    }

    #[test]
    fn test_verify_page() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_binary");
        let data = vec![0xABu8; PAGE_SIZE * 3];
        std::fs::write(&path, &data).unwrap();

        let mut integrity = BinaryIntegrity::new(path.to_string_lossy().to_string());
        let manifest = integrity.generate_manifest("1.0.0").unwrap();
        integrity.load_manifest(manifest);

        assert!(integrity.verify_page(0).unwrap());
        assert!(integrity.verify_page(1).unwrap());
        assert!(integrity.verify_page(2).unwrap());
    }
}
