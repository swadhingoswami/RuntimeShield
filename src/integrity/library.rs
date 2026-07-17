use crate::core::error::{Error, Result};
use crate::crypto::hash::hash_file;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LibraryEntry {
    pub name: String,
    pub path: String,
    pub hash: String,
}

#[derive(Clone)]
pub struct LibraryIntegrity {
    loaded_libraries: Vec<LibraryEntry>,
    manifest_libraries: HashMap<String, LibraryEntry>,
}

impl LibraryIntegrity {
    pub fn new() -> Self {
        Self {
            loaded_libraries: Vec::new(),
            manifest_libraries: HashMap::new(),
        }
    }

    pub fn load_manifest(&mut self, entries: Vec<LibraryEntry>) {
        for entry in entries {
            self.manifest_libraries.insert(entry.name.clone(), entry);
        }
    }

    pub fn enumerate_loaded_libraries(&mut self) -> Result<()> {
        let libraries = enumerate_shared_libraries()?;
        self.loaded_libraries = libraries;
        Ok(())
    }

    pub fn verify_all(&self) -> Result<Vec<LibraryEntry>> {
        let mut mismatches = Vec::new();

        for lib in &self.loaded_libraries {
            if let Some(expected) = self.manifest_libraries.get(&lib.name) {
                let actual_hash = hash_file(Path::new(&lib.path)).map_err(Error::Io)?;
                let actual_hex = hex::encode(actual_hash);
                if actual_hex != expected.hash {
                    mismatches.push(LibraryEntry {
                        name: lib.name.clone(),
                        path: lib.path.clone(),
                        hash: actual_hex,
                    });
                }
            }
        }

        Ok(mismatches)
    }

    pub fn loaded_libraries(&self) -> &[LibraryEntry] {
        &self.loaded_libraries
    }
}

impl Default for LibraryIntegrity {
    fn default() -> Self {
        Self::new()
    }
}

fn enumerate_shared_libraries() -> Result<Vec<LibraryEntry>> {
    #[cfg(target_os = "linux")]
    {
        enumerate_linux_libraries()
    }
    #[cfg(target_os = "macos")]
    {
        enumerate_macos_libraries()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(Error::Platform(
            "library enumeration not supported on this platform".into(),
        ))
    }
}

#[cfg(target_os = "linux")]
fn enumerate_linux_libraries() -> Result<Vec<LibraryEntry>> {
    let maps = std::fs::read_to_string("/proc/self/maps")
        .map_err(|e| Error::Platform(format!("failed to read /proc/self/maps: {}", e)))?;

    let mut seen = std::collections::HashSet::new();
    let mut libraries = Vec::new();

    for line in maps.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let path = parts[5];
            if ((path.starts_with('/') && path.ends_with(".so")) || path.contains(".so."))
                && seen.insert(path.to_string())
            {
                let name = Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string());

                let hash = match hash_file(Path::new(path)) {
                    Ok(h) => hex::encode(h),
                    Err(e) => {
                        log::warn!("failed to hash library '{}': {}", path, e);
                        continue;
                    }
                };

                libraries.push(LibraryEntry {
                    name,
                    path: path.to_string(),
                    hash,
                });
            }
        }
    }

    Ok(libraries)
}

#[cfg(target_os = "macos")]
fn enumerate_macos_libraries() -> Result<Vec<LibraryEntry>> {
    let count = unsafe { _dyld_image_count() };
    let mut seen = std::collections::HashSet::new();
    let mut libraries = Vec::new();

    for i in 0..count {
        let name_ptr = unsafe { _dyld_get_image_name(i) };
        let _header_ptr = unsafe { _dyld_get_image_header(i) };

        if name_ptr.is_null() {
            continue;
        }

        let path = unsafe { CStr::from_ptr(name_ptr) }
            .to_string_lossy()
            .into_owned();

        if path.is_empty() || !seen.insert(path.clone()) {
            continue;
        }

        let name = Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());

        let hash = match hash_file(Path::new(&path)) {
            Ok(h) => hex::encode(h),
            Err(e) => {
                log::warn!("failed to hash macOS library '{}': {}", path, e);
                continue;
            }
        };

        libraries.push(LibraryEntry { name, path, hash });
    }

    Ok(libraries)
}

#[cfg(target_os = "macos")]
use std::ffi::CStr;

extern "C" {
    fn _dyld_image_count() -> u32;
    fn _dyld_get_image_name(index: u32) -> *const libc::c_char;
    fn _dyld_get_image_header(index: u32) -> *const libc::c_void;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_library_integrity() {
        let li = LibraryIntegrity::new();
        assert!(li.loaded_libraries().is_empty());
    }

    #[test]
    fn test_manifest_matching() {
        let mut li = LibraryIntegrity::new();
        let entries = vec![LibraryEntry {
            name: "libtest.so".into(),
            path: "/usr/lib/libtest.so".into(),
            hash: "abc123".into(),
        }];
        li.load_manifest(entries);
        assert_eq!(li.manifest_libraries.len(), 1);
    }
}
