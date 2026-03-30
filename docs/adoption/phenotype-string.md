# phenotype-string Adoption Guide

## Overview

`phenotype-string` provides canonical string manipulation utilities.

## Quick Start

### Add Dependency

```toml
[dependencies]
phenotype-string = { path = "../crates/phenotype-string" }
```

## Sanitization

```rust
use phenotype_string::sanitize::sanitize_filename;

let safe = sanitize_filename("test<>:file.txt");
// Result: "test___file.txt"
```

## Identifier Parsing

```rust
use phenotype_string::parse::{parse_kebab_case, parse_snake_case};

let words = parse_kebab_case("my-component-name")?;
// Result: ["my", "component", "name"]

let words = parse_snake_case("my_component_name")?;
// Result: ["my", "component", "name"]
```

## Joining

```rust
use phenotype_string::join::join_with_prefix;

let result = join_with_prefix(&["a", "b", "c"], "prefix-");
// Result: "prefix-a, prefix-b, prefix-c"
```

## Related Crates

- `phenotype-iter` - Iterator extensions
