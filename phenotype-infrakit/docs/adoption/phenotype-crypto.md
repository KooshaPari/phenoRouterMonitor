# phenotype-crypto Adoption Guide

## Overview

`phenotype-crypto` provides canonical hashing, encoding, and randomness utilities.

## Quick Start

### Add Dependency

```toml
[dependencies]
phenotype-crypto = { path = "../crates/phenotype-crypto" }
```

## Hash Chain

```rust
use phenotype_crypto::hash::{HashChain, Hashable};

#[derive(Debug, Clone)]
struct Event {
    id: String,
    data: String,
    timestamp: i64,
}

impl Hashable for Event {
    fn hash_content(&self) -> Vec<u8> {
        format!("{}:{}:{}", self.id, self.data, self.timestamp).into_bytes()
    }
}

let chain = HashChain::new();
let hash1 = chain.add(Event { id: "1".into(), data: "test".into(), timestamp: 100 });
let hash2 = chain.add(Event { id: "2".into(), data: "test2".into(), timestamp: 200 });
```

## SHA-256 Hashing

```rust
use phenotype_crypto::hash::{sha256_hash, verify_sha256};

let data = b"Hello, World!";
let hash = sha256_hash(data);
// Returns 32-byte hash

let valid = verify_sha256(data, &hash);
```

## Hex Encoding

```rust
use phenotype_crypto::hex::{encode_hex, decode_hex};

let bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
let hex_string = encode_hex(&bytes);
// Returns "48656c6c6f"

let decoded = decode_hex("48656c6c6f").unwrap();
// Returns [0x48, 0x65, 0x6c, 0x6c, 0x6f]
```

## Random Utilities

```rust
use phenotype_crypto::random::generate_random_id;

let id: String = generate_random_id();
// Returns 21-character URL-safe base64 ID
```

## Related Crates

- `phenotype-error-core` - Error types for crypto failures
- `phenotype-time` - Timestamp utilities for hash chains
