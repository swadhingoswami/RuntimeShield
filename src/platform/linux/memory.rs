use crate::core::error::{Error, Result};
use crate::platform::MemoryRegionReader;

pub struct LinuxMemory;

impl LinuxMemory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryRegionReader for LinuxMemory {
    fn read_region(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        use std::fs::File;
        use std::os::unix::fs::FileExt;

        let mem_path = format!("/proc/{}/mem", std::process::id());
        let file = File::open(&mem_path)
            .map_err(|e| Error::Platform(format!("failed to open {}: {}", mem_path, e)))?;
        let mut buf = vec![0u8; size];
        file.read_exact_at(&mut buf, address as u64)
            .map_err(|e| Error::Platform(format!("failed to read memory at {:#x}: {}", address, e)))?;
        Ok(buf)
    }

    fn get_code_regions(&self) -> Result<Vec<(usize, usize)>> {
        let maps_content = std::fs::read_to_string("/proc/self/maps")
            .map_err(|e| Error::Platform(format!("failed to read /proc/self/maps: {}", e)))?;

        let mut regions = Vec::new();
        for line in maps_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }
            let perms = parts[1];
            // Only read-execute or read-only executable regions
            if perms.contains('x') {
                let addr_range: Vec<&str> = parts[0].split('-').collect();
                if addr_range.len() == 2 {
                    if let (Ok(start), Ok(end)) =
                        (usize::from_str_radix(addr_range[0], 16), usize::from_str_radix(addr_range[1], 16))
                    {
                        regions.push((start, end - start));
                    }
                }
            }
        }

        Ok(regions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_code_regions() {
        let mem = LinuxMemory::new();
        match mem.get_code_regions() {
            Ok(regions) => {
                assert!(!regions.is_empty(), "should have at least one code region");
                for (_, size) in &regions {
                    assert!(*size > 0);
                }
            }
            Err(e) => {
                // On systems without /proc, skip
                assert!(
                    e.to_string().contains("/proc"),
                    "Unexpected error: {}",
                    e
                );
            }
        }
    }
}
