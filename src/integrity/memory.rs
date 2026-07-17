use crate::core::error::{Error, Result};
use crate::crypto::hash::hash_bytes;
use crate::platform::PlatformMemory;
use crate::platform::MemoryRegionReader;

#[derive(Clone)]
pub struct MemoryIntegrity {
    memory_reader: PlatformMemory,
    protected_regions: Vec<(usize, usize)>, // (address, size)
    known_hashes: Vec<String>,
}

impl MemoryIntegrity {
    pub fn new() -> Self {
        Self {
            memory_reader: PlatformMemory::new(),
            protected_regions: Vec::new(),
            known_hashes: Vec::new(),
        }
    }

    pub fn add_protected_region(&mut self, address: usize, size: usize) {
        self.protected_regions.push((address, size));
    }

    pub fn auto_discover_code_regions(&mut self) -> Result<()> {
        let regions = self.memory_reader.get_code_regions()?;
        for (addr, size) in regions {
            self.protected_regions.push((addr, size));
        }
        Ok(())
    }

    pub fn snapshot_hashes(&mut self) -> Result<()> {
        self.known_hashes.clear();
        for (addr, size) in &self.protected_regions {
            let data = self.memory_reader.read_region(*addr, *size)?;
            let hash = hash_bytes(&data);
            self.known_hashes.push(hex::encode(hash));
        }
        Ok(())
    }

pub fn verify_all(&self) -> Result<Vec<usize>> {
    if self.protected_regions.is_empty() {
        return Err(Error::Verification("no protected regions configured".into()));
    }
    if self.known_hashes.len() != self.protected_regions.len() {
        return Err(Error::Verification(
            "number of known hashes does not match protected regions; call snapshot_hashes first"
                .into(),
        ));
    }

        let mut modified_indices = Vec::new();

        for (i, (addr, size)) in self.protected_regions.iter().enumerate() {
            let data = self.memory_reader.read_region(*addr, *size)?;
            let hash = hash_bytes(&data);
            let hex_hash = hex::encode(hash);

            if hex_hash != self.known_hashes[i] {
                modified_indices.push(i);
            }
        }

        Ok(modified_indices)
    }

    pub fn protected_regions(&self) -> &[(usize, usize)] {
        &self.protected_regions
    }
}

impl Default for MemoryIntegrity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_integrity() {
        let mi = MemoryIntegrity::new();
        assert!(mi.protected_regions().is_empty());
    }

    #[test]
    fn test_add_protected_region() {
        let mut mi = MemoryIntegrity::new();
        mi.add_protected_region(0x1000, 4096);
        assert_eq!(mi.protected_regions().len(), 1);
    }

    #[test]
    fn test_verify_without_snapshot_fails() {
        let mi = MemoryIntegrity::new();
        let result = mi.verify_all();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("no protected regions") || err.contains("snapshot") || err.contains("known hash"));
    }
}
