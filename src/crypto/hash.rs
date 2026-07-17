use sha2::{Digest, Sha256};
use std::io::Read;

pub const HASH_SIZE: usize = 32;
pub type HashValue = [u8; HASH_SIZE];

pub fn hash_bytes(data: &[u8]) -> HashValue {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; HASH_SIZE];
    out.copy_from_slice(&result);
    out
}

pub fn hash_file(path: &std::path::Path) -> std::io::Result<HashValue> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let result = hasher.finalize();
    let mut out = [0u8; HASH_SIZE];
    out.copy_from_slice(&result);
    Ok(out)
}

pub fn hash_reader<R: Read>(mut reader: R) -> std::io::Result<HashValue> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let result = hasher.finalize();
    let mut out = [0u8; HASH_SIZE];
    out.copy_from_slice(&result);
    Ok(out)
}

pub fn hash_to_hex(hash: &HashValue) -> String {
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes_deterministic() {
        let data = b"hello world";
        let h1 = hash_bytes(data);
        let h2 = hash_bytes(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_bytes_different() {
        let h1 = hash_bytes(b"hello");
        let h2 = hash_bytes(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_size() {
        let hash = hash_bytes(b"test");
        assert_eq!(hash.len(), HASH_SIZE);
    }

    #[test]
    fn test_hash_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"file content").unwrap();
        let hash = hash_file(&path).unwrap();
        assert_eq!(hash.len(), HASH_SIZE);
    }

    #[test]
    fn test_hash_to_hex() {
        let hash = hash_bytes(b"test");
        let hex = hash_to_hex(&hash);
        assert_eq!(hex.len(), 64);
    }
}
