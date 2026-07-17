use crate::crypto::hash::{hash_bytes, HashValue, HASH_SIZE};

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: HashValue,
    pub children: Vec<MerkleNode>,
}

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: MerkleNode,
    pub leaf_count: usize,
    pub levels: usize,
    pub page_hashes: Vec<HashValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ManifestEntry {
    pub page_index: usize,
    pub page_hash: String,
    pub offset: u64,
    pub size: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub root_hash: String,
    pub entries: Vec<ManifestEntry>,
    pub total_pages: usize,
    pub file_size: u64,
    pub version: String,
    pub timestamp: String,
}

pub fn build_merkle_tree(data: &[u8]) -> MerkleTree {
    let leaf_hashes: Vec<HashValue> = data.chunks(PAGE_SIZE).map(hash_bytes).collect();

    let leaf_count = leaf_hashes.len();
    if leaf_count == 0 {
        return MerkleTree {
            root: MerkleNode {
                hash: hash_bytes(b""),
                children: vec![],
            },
            leaf_count: 0,
            levels: 0,
            page_hashes: vec![],
        };
    }

    let mut current_level: Vec<MerkleNode> = leaf_hashes
        .iter()
        .map(|hash| MerkleNode {
            hash: *hash,
            children: vec![],
        })
        .collect();

    let mut levels = 1;
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        for chunk in current_level.chunks(2) {
            if chunk.len() == 2 {
                let mut combined = Vec::with_capacity(HASH_SIZE * 2);
                combined.extend_from_slice(&chunk[0].hash);
                combined.extend_from_slice(&chunk[1].hash);
                let parent_hash = hash_bytes(&combined);
                next_level.push(MerkleNode {
                    hash: parent_hash,
                    children: vec![chunk[0].clone(), chunk[1].clone()],
                });
            } else {
                next_level.push(chunk[0].clone());
            }
        }
        current_level = next_level;
        levels += 1;
    }

    MerkleTree {
        root: current_level.into_iter().next().unwrap(),
        leaf_count,
        levels,
        page_hashes: leaf_hashes,
    }
}

pub fn verify_page_hash(tree: &MerkleTree, data: &[u8], page_index: usize) -> bool {
    if page_index >= tree.leaf_count {
        return false;
    }
    let actual_hash = hash_bytes(data);
    actual_hash == tree.page_hashes[page_index]
}

pub fn get_page_hash(tree: &MerkleTree, page_index: usize) -> Option<HashValue> {
    if page_index >= tree.leaf_count {
        return None;
    }
    Some(tree.page_hashes[page_index])
}

pub fn build_manifest(data: &[u8], version: &str) -> Manifest {
    let tree = build_merkle_tree(data);
    let entries: Vec<ManifestEntry> = data
        .chunks(PAGE_SIZE)
        .enumerate()
        .map(|(i, chunk)| ManifestEntry {
            page_index: i,
            page_hash: hex::encode(get_page_hash(&tree, i).unwrap_or([0u8; HASH_SIZE])),
            offset: (i * PAGE_SIZE) as u64,
            size: chunk.len(),
        })
        .collect();

    Manifest {
        root_hash: hex::encode(tree.root.hash),
        entries,
        total_pages: tree.leaf_count,
        file_size: data.len() as u64,
        version: version.to_string(),
        timestamp: chrono_now(),
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_data() {
        let tree = build_merkle_tree(b"");
        assert_eq!(tree.leaf_count, 0);
    }

    #[test]
    fn test_single_page() {
        let data = vec![0u8; 100];
        let tree = build_merkle_tree(&data);
        assert_eq!(tree.leaf_count, 1);
        assert_eq!(tree.levels, 1);
    }

    #[test]
    fn test_multi_page() {
        let data = vec![0u8; PAGE_SIZE * 4 + 100];
        let tree = build_merkle_tree(&data);
        assert_eq!(tree.leaf_count, 5);
        assert!(tree.levels > 1);
    }

    #[test]
    fn test_verify_page_valid() {
        let data = vec![0xABu8; PAGE_SIZE * 2];
        let tree = build_merkle_tree(&data);
        assert!(verify_page_hash(&tree, &data[..PAGE_SIZE], 0));
        assert!(verify_page_hash(&tree, &data[PAGE_SIZE..], 1));
    }

    #[test]
    fn test_verify_page_invalid() {
        let data = vec![0xABu8; PAGE_SIZE];
        let tree = build_merkle_tree(&data);
        assert!(!verify_page_hash(&tree, &[0x00u8; PAGE_SIZE], 0));
    }

    #[test]
    fn test_manifest_roundtrip() {
        let data = vec![0x42u8; PAGE_SIZE * 3 + 500];
        let manifest = build_manifest(&data, "1.0.0");
        assert_eq!(manifest.total_pages, 4);
        assert!(!manifest.root_hash.is_empty());
        assert_eq!(manifest.entries.len(), 4);
        assert_eq!(manifest.version, "1.0.0");
    }

    #[test]
    fn test_deterministic_manifest() {
        let data = b"deterministic test data";
        let m1 = build_manifest(data, "1.0");
        let m2 = build_manifest(data, "1.0");
        assert_eq!(m1.root_hash, m2.root_hash);
        for (e1, e2) in m1.entries.iter().zip(m2.entries.iter()) {
            assert_eq!(e1.page_hash, e2.page_hash);
        }
    }
}
