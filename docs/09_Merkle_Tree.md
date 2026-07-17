# Merkle Tree

## Overview

A Merkle tree (also known as a hash tree) is a data structure that enables efficient and secure verification of large data sets. RuntimeShield uses Merkle trees for binary integrity verification.

## Structure

```mermaid
graph TB
    subgraph "Level 2 - Root"
        R[Root Hash<br/>hash(H0 + H1)]
    end
    
    subgraph "Level 1 - Internal Nodes"
        H0[Hash 0-1<br/>hash(L0 + L1)]
        H1[Hash 2-3<br/>hash(L2 + L3)]
    end
    
    subgraph "Level 0 - Leaves"
        L0[Leaf 0<br/>hash(Page 0)]
        L1[Leaf 1<br/>hash(Page 1)]
        L2[Leaf 2<br/>hash(Page 2)]
        L3[Leaf 3<br/>hash(Page 3)]
    end
    
    subgraph "Data"
        P0[Page 0<br/>bytes 0-4095]
        P1[Page 1<br/>bytes 4096-8191]
        P2[Page 2<br/>bytes 8192-12287]
        P3[Page 3<br/>bytes 12288-16383]
    end
    
    R --> H0
    R --> H1
    H0 --> L0
    H0 --> L1
    H1 --> L2
    H1 --> L3
    L0 --> P0
    L1 --> P1
    L2 --> P2
    L3 --> P3
```

## How It Works

1. **Split data into pages** — The binary is divided into fixed-size pages (4096 bytes each).

2. **Hash each page** — Each page is hashed using SHA-256, producing leaf hashes.

3. **Pair and hash** — Leaf hashes are paired and concatenated, then hashed to produce parent hashes.

4. **Repeat** — The process continues up the tree until a single root hash remains.

5. **Verify** — To verify a specific page, hash just that page and compare with the stored leaf hash. To verify the entire binary, compute the root hash and compare.

## Why Merkle Trees?

### Efficiency

- **Full verification**: O(n) — hash all pages → O(log n) parent hashes
- **Page verification**: O(1) — hash single page and compare with leaf hash
- **Storage**: Linear in file size (one hash per page + internal nodes)

### Properties

| Property | Description |
|---|---|
| **Completeness** | Root hash depends on all leaf hashes |
| **Soundness** | Impossible to forge a valid root hash for modified data |
| **Locality** | A single page modification changes only O(log n) hashes |
| **Efficiency** | Each hash is small (32 bytes for SHA-256) |

## Implementation

```rust
use runtimeshield::crypto::merkle::{build_merkle_tree, verify_page_hash};

let data = std::fs::read("app.exe")?;
let tree = build_merkle_tree(&data);

println!("Pages: {}", tree.leaf_count);
println!("Levels: {}", tree.levels);
println!("Root hash: {}", hex::encode(tree.root.hash));

// Verify a specific page
let page_data = &data[0..4096];
assert!(verify_page_hash(&tree, page_data, 0));
```

## Storage Efficiency

For a binary of size S with page size P and hash size H (32 bytes for SHA-256):

| Binary Size | Pages | Leaf Hashes | Internal Hashes | Total Storage |
|---|---|---|---|---|
| 1 MB | 256 | 8 KB | ~8 KB | ~16 KB |
| 10 MB | 2,560 | 80 KB | ~80 KB | ~160 KB |
| 100 MB | 25,600 | 800 KB | ~800 KB | ~1.6 MB |
| 1 GB | 262,144 | 8 MB | ~8 MB | ~16 MB |

## Comparison with Simple Hashing

### Simple Hash (SHA-256 of entire file)

```
hash(entire_file) = 32 bytes
```

**Pros**: Simple, fast to compute, small storage.
**Cons**: Cannot determine which page changed; must re-hash entire file for verification.

### Merkle Tree (page-level)

```
hash(page_0) + hash(page_1) + ... + internal_nodes + root_hash
```

**Pros**: Page-level granularity, efficient partial verification.
**Cons**: More storage, slightly more computation for tree construction.

## Limitations

1. **Page size tradeoff**: Smaller pages give finer granularity but more hashes. 4096 bytes is a good default.

2. **No tamper resistance**: The Merkle tree itself must be protected. A Merkle tree can verify integrity but cannot prevent modification.

3. **Perfect binary tree assumption**: Real data produces unbalanced trees when the page count is not a power of two. RuntimeShield handles this by promoting orphaned nodes.

4. **Hash collision resistance**: Security depends on SHA-256 collision resistance. Currently considered secure.
