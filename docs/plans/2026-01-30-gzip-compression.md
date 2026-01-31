# Gzip Compression Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add transparent gzip compression/decompression for `.json.gz` and `.jsonl.gz` files

**Architecture:** Isolated changes to file I/O layer (`loader.rs` and `saver.rs`). Detect `.gz` extension, decompress on load, optionally compress on save. All existing features (atomic writes, backups, format preservation) work unchanged.

**Tech Stack:** Rust, flate2 (gzip), serde_json, tempfile (for tests)

---

## Task 1: Add flate2 Dependency

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add flate2 dependency**

Add to `[dependencies]` section in `Cargo.toml`:

```toml
flate2 = "1.0"
```

**Step 2: Verify dependency resolves**

Run: `cargo build`
Expected: Compiles successfully, flate2 downloaded

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add flate2 dependency for gzip support"
```

---

## Task 2: Add Helper Function - determine_jsonl_format

**Files:**
- Modify: `src/file/loader.rs:60` (after `load_jsonl_file` function)
- Test: `src/file/loader.rs:147` (in `#[cfg(test)] mod tests`)

**Step 1: Write the failing test**

Add to test module in `src/file/loader.rs`:

```rust
#[test]
fn test_determine_jsonl_format() {
    assert!(determine_jsonl_format("data.jsonl"));
    assert!(determine_jsonl_format("data.ndjson"));
    assert!(determine_jsonl_format("path/to/data.jsonl.gz"));
    assert!(determine_jsonl_format("path/to/data.ndjson.gz"));
    assert!(!determine_jsonl_format("data.json"));
    assert!(!determine_jsonl_format("data.json.gz"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_determine_jsonl_format`
Expected: FAIL with "cannot find function `determine_jsonl_format`"

**Step 3: Write minimal implementation**

Add after `load_jsonl_file` function in `src/file/loader.rs`:

```rust
/// Determines if file is JSONL format based on filename.
///
/// Checks for .jsonl or .ndjson extension, handling .gz suffix correctly.
/// Examples:
/// - `data.jsonl` → true
/// - `data.jsonl.gz` → true
/// - `data.json.gz` → false
fn determine_jsonl_format<P: AsRef<Path>>(path: P) -> bool {
    let path_str = path.as_ref().to_string_lossy();

    // Remove .gz suffix if present
    let base = if path_str.ends_with(".gz") {
        &path_str[..path_str.len() - 3]
    } else {
        &path_str
    };

    base.ends_with(".jsonl") || base.ends_with(".ndjson")
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_determine_jsonl_format`
Expected: PASS

**Step 5: Commit**

```bash
git add src/file/loader.rs
git commit -m "feat: add determine_jsonl_format helper function"
```

---

## Task 3: Add Helper Function - read_gzipped_file

**Files:**
- Modify: `src/file/loader.rs:75` (after `determine_jsonl_format`)
- Test: `src/file/loader.rs` (test module)

**Step 1: Write the failing test**

Add to test module in `src/file/loader.rs`:

```rust
#[test]
fn test_read_gzipped_file() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create temp file with gzipped JSON
    let json_content = r#"{"test": "value"}"#;
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");

    // Write compressed content
    let file = fs::File::create(&gz_path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(json_content.as_bytes()).unwrap();
    encoder.finish().unwrap();

    // Test decompression
    let decompressed = read_gzipped_file(&gz_path).unwrap();
    assert_eq!(decompressed, json_content);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_read_gzipped_file`
Expected: FAIL with "cannot find function `read_gzipped_file`"

**Step 3: Write minimal implementation**

Add after `determine_jsonl_format` in `src/file/loader.rs`:

```rust
/// Reads and decompresses a gzipped file.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened
/// - The file is not valid gzip format (corrupted)
/// - The decompressed content is not valid UTF-8
fn read_gzipped_file<P: AsRef<Path>>(path: P) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let file = fs::File::open(path).context("Failed to open gzipped file")?;
    let mut decoder = GzDecoder::new(file);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .context("Failed to decompress gzipped file - file may be corrupted")?;
    Ok(content)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_read_gzipped_file`
Expected: PASS

**Step 5: Commit**

```bash
git add src/file/loader.rs
git commit -m "feat: add read_gzipped_file helper function"
```

---

## Task 4: Add Test for Corrupted Gzip File

**Files:**
- Test: `src/file/loader.rs` (test module)

**Step 1: Write the failing test**

Add to test module in `src/file/loader.rs`:

```rust
#[test]
fn test_read_gzipped_file_corrupted() {
    use tempfile::NamedTempFile;

    // Create file with .gz extension but invalid gzip data
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");
    fs::write(&gz_path, b"not gzip data").unwrap();

    // Should return error with helpful message
    let result = read_gzipped_file(&gz_path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("decompress") || err_msg.contains("corrupted"));
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test test_read_gzipped_file_corrupted`
Expected: PASS (implementation already handles this)

**Step 3: Commit**

```bash
git add src/file/loader.rs
git commit -m "test: add corrupted gzip file error handling test"
```

---

## Task 5: Modify load_json_file to Support Gzip

**Files:**
- Modify: `src/file/loader.rs:44-58` (replace `load_json_file` function)
- Test: `src/file/loader.rs` (test module)

**Step 1: Write the failing test**

Add to test module in `src/file/loader.rs`:

```rust
#[test]
fn test_load_gzipped_json_file() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create temp file with gzipped JSON
    let json_content = r#"{"name": "Alice", "age": 30}"#;
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");

    // Write compressed content
    let file = fs::File::create(&gz_path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(json_content.as_bytes()).unwrap();
    encoder.finish().unwrap();

    // Load and verify
    let tree = load_json_file(&gz_path).unwrap();

    // Verify structure
    use crate::document::node::JsonValue;
    if let JsonValue::Object(entries) = tree.root().value() {
        assert_eq!(entries.len(), 2);
    } else {
        panic!("Expected object");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_load_gzipped_json_file`
Expected: FAIL (current implementation doesn't handle .gz)

**Step 3: Modify load_json_file implementation**

Replace `load_json_file` function in `src/file/loader.rs`:

```rust
pub fn load_json_file<P: AsRef<Path>>(path: P) -> Result<JsonTree> {
    let path_ref = path.as_ref();

    // Check if file is gzipped
    let is_gzipped = path_ref
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "gz")
        .unwrap_or(false);

    // Read content (decompress if needed)
    let content = if is_gzipped {
        read_gzipped_file(path_ref)?
    } else {
        fs::read_to_string(path_ref).context("Failed to read file")?
    };

    // Determine format from filename (before .gz)
    let is_jsonl = determine_jsonl_format(path_ref);

    // Parse accordingly
    if is_jsonl {
        parse_jsonl_content(&content).context("Failed to parse JSONL")
    } else {
        parse_json(&content).context("Failed to parse JSON")
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_load_gzipped_json_file`
Expected: PASS

**Step 5: Run all loader tests**

Run: `cargo test --lib file::loader`
Expected: All tests PASS

**Step 6: Commit**

```bash
git add src/file/loader.rs
git commit -m "feat: add gzip decompression support to load_json_file"
```

---

## Task 6: Add Test for Gzipped JSONL Files

**Files:**
- Test: `src/file/loader.rs` (test module)

**Step 1: Write the failing test**

Add to test module in `src/file/loader.rs`:

```rust
#[test]
fn test_load_gzipped_jsonl_file() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create temp file with gzipped JSONL
    let jsonl_content = r#"{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
{"id":3,"name":"Charlie"}"#;
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("jsonl.gz");

    // Write compressed content
    let file = fs::File::create(&gz_path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(jsonl_content.as_bytes()).unwrap();
    encoder.finish().unwrap();

    // Load and verify
    let tree = load_json_file(&gz_path).unwrap();

    // Verify it's JSONL format
    use crate::document::node::JsonValue;
    if let JsonValue::JsonlRoot(lines) = tree.root().value() {
        assert_eq!(lines.len(), 3);
    } else {
        panic!("Expected JsonlRoot");
    }
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test test_load_gzipped_jsonl_file`
Expected: PASS (implementation already handles this via determine_jsonl_format)

**Step 3: Commit**

```bash
git add src/file/loader.rs
git commit -m "test: add gzipped JSONL file loading test"
```

---

## Task 7: Add Stdin Gzip Auto-Detection

**Files:**
- Modify: `src/file/loader.rs:120-136` (replace `load_json_from_stdin`)
- Add: `src/file/loader.rs` (new `decompress_gzip_bytes` function)

**Step 1: Add decompress_gzip_bytes helper**

Add after `read_gzipped_file` in `src/file/loader.rs`:

```rust
/// Decompresses gzip-encoded bytes to a UTF-8 string.
///
/// # Errors
///
/// Returns an error if:
/// - The bytes are not valid gzip format
/// - The decompressed content is not valid UTF-8
fn decompress_gzip_bytes(bytes: &[u8]) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(bytes);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .context("Failed to decompress gzipped stdin")?;
    Ok(content)
}
```

**Step 2: Modify load_json_from_stdin**

Replace `load_json_from_stdin` in `src/file/loader.rs`:

```rust
pub fn load_json_from_stdin() -> Result<JsonTree> {
    use std::io::{self, Read};

    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .context("Failed to read from stdin")?;

    // Check for gzip magic bytes (0x1f 0x8b)
    let content = if buffer.starts_with(&[0x1f, 0x8b]) {
        decompress_gzip_bytes(&buffer)?
    } else {
        String::from_utf8(buffer).context("Invalid UTF-8 in stdin")?
    };

    // Try to parse as regular JSON first
    if let Ok(tree) = parse_json(&content) {
        return Ok(tree);
    }

    // If regular JSON parsing fails, try JSONL format
    parse_jsonl_content(&content).context(
        "Failed to parse JSON from stdin: input is neither valid JSON nor valid JSONL",
    )
}
```

**Step 3: Run existing tests**

Run: `cargo test --lib file::loader`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/file/loader.rs
git commit -m "feat: add gzip auto-detection for stdin input"
```

---

## Task 8: Add Helper Function - write_file_atomic

**Files:**
- Add: `src/file/saver.rs:112` (after `save_json_file` function, before `save_jsonl`)
- Test: `src/file/saver.rs:572` (in test module)

**Step 1: Write the failing test**

Add to test module in `src/file/saver.rs`:

```rust
#[test]
fn test_write_file_atomic_uncompressed() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let target_path = temp_file.path();
    let data = b"test content";

    write_file_atomic(target_path, data, false).unwrap();

    let written = fs::read_to_string(target_path).unwrap();
    assert_eq!(written, "test content");
}

#[test]
fn test_write_file_atomic_compressed() {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let target_path = temp_file.path().with_extension("json.gz");
    let data = b"test content";

    write_file_atomic(&target_path, data, true).unwrap();

    // Decompress and verify
    let file = fs::File::open(&target_path).unwrap();
    let mut decoder = GzDecoder::new(file);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).unwrap();
    assert_eq!(decompressed, "test content");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_write_file_atomic`
Expected: FAIL with "cannot find function `write_file_atomic`"

**Step 3: Write implementation**

Add after current `save_json_file` function in `src/file/saver.rs`:

```rust
/// Writes data to a file atomically, optionally compressing with gzip.
///
/// This function writes to a temporary file first, then atomically renames
/// it to the target path. This ensures the target file is never left in a
/// partially written state.
///
/// # Arguments
///
/// * `path` - Target file path
/// * `data` - Bytes to write
/// * `compress` - Whether to gzip-compress the data before writing
///
/// # Errors
///
/// Returns an error if:
/// - Creating the temp file fails
/// - Writing or compressing fails
/// - Renaming the temp file fails
fn write_file_atomic<P: AsRef<Path>>(path: P, data: &[u8], compress: bool) -> Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let path = path.as_ref();
    let temp_path = path.with_extension("tmp");

    if compress {
        // Write compressed
        let file = fs::File::create(&temp_path).context("Failed to create temp file")?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder
            .write_all(data)
            .context("Failed to write compressed data")?;
        encoder
            .finish()
            .context("Failed to finish compression")?;
    } else {
        // Write uncompressed
        fs::write(&temp_path, data).context("Failed to write temp file")?;
    }

    // Atomic rename
    fs::rename(&temp_path, path).context("Failed to rename temp file")?;

    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_write_file_atomic`
Expected: PASS

**Step 5: Commit**

```bash
git add src/file/saver.rs
git commit -m "feat: add write_file_atomic with optional compression"
```

---

## Task 9: Modify save_json_file to Support Gzip

**Files:**
- Modify: `src/file/saver.rs:64-111` (replace `save_json_file` function)
- Modify: `src/file/saver.rs:116-155` (update `save_jsonl` signature)

**Step 1: Extract create_backup helper (preparation)**

Add helper function before `save_json_file` in `src/file/saver.rs`:

```rust
/// Creates a backup of a file by copying it with a .bak extension.
fn create_backup<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let mut backup_path = path.to_path_buf();
    let original_name = backup_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
    backup_path.set_file_name(format!("{}.bak", original_name));
    fs::copy(path, backup_path).context("Failed to create backup")?;
    Ok(())
}
```

**Step 2: Modify save_json_file**

Replace `save_json_file` function in `src/file/saver.rs`:

```rust
pub fn save_json_file<P: AsRef<Path>>(path: P, tree: &JsonTree, config: &Config) -> Result<()> {
    let path = path.as_ref();

    // Determine if we should compress based on target filename
    let should_compress = path.to_string_lossy().ends_with(".gz");

    // Check if this is a JSONL document
    if matches!(tree.root().value(), JsonValue::JsonlRoot(_)) {
        return save_jsonl(path, tree, config, should_compress);
    }

    // Create backup if requested and file exists
    if config.create_backup && path.exists() {
        create_backup(path)?;
    }

    // Serialize with format preservation if original source is available
    let mut json_str = if let Some(original) = tree.original_source() {
        serialize_preserving_format(tree.root(), original, config, 0)
    } else {
        // No original source, use standard serialization
        serialize_node(tree.root(), config.indent_size, 0)
    };

    // Preserve trailing newline from original if present
    if let Some(original) = tree.original_source() {
        if original.ends_with('\n') && !json_str.ends_with('\n') {
            json_str.push('\n');
        }
    }

    // Validate the serialized JSON before writing to disk
    // This catches serialization bugs before they corrupt user data
    serde_json::from_str::<serde_json::Value>(&json_str)
        .context("Generated invalid JSON - this is a bug in jsonquill's serialization")?;

    // Write atomically (compressed or uncompressed)
    write_file_atomic(path, json_str.as_bytes(), should_compress)?;

    Ok(())
}
```

**Step 3: Update save_jsonl signature**

Update `save_jsonl` function signature in `src/file/saver.rs`:

```rust
fn save_jsonl<P: AsRef<Path>>(
    path: P,
    tree: &JsonTree,
    config: &Config,
    compress: bool,
) -> Result<()> {
    let path = path.as_ref();

    // Create backup if requested and file exists
    if config.create_backup && path.exists() {
        create_backup(path)?;
    }

    let mut output = String::new();

    if let JsonValue::JsonlRoot(lines) = tree.root().value() {
        for (i, node) in lines.iter().enumerate() {
            // JSONL requires compact single-line JSON
            let line = serialize_node_compact(node);

            // Validate each line is valid JSON
            serde_json::from_str::<serde_json::Value>(&line).with_context(|| {
                format!(
                    "Generated invalid JSON at line {} - this is a bug in jsonquill's serialization",
                    i + 1
                )
            })?;

            output.push_str(&line);
            output.push('\n');
        }
    }

    // Write atomically with optional compression
    write_file_atomic(path, output.as_bytes(), compress)?;

    Ok(())
}
```

**Step 4: Run existing tests**

Run: `cargo test --lib file::saver`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/file/saver.rs
git commit -m "feat: add gzip compression support to save functions"
```

---

## Task 10: Add Saver Tests for Gzip

**Files:**
- Test: `src/file/saver.rs` (test module)

**Step 1: Add test for saving gzipped JSON**

Add to test module in `src/file/saver.rs`:

```rust
#[test]
fn test_save_json_as_gzipped() {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tempfile::NamedTempFile;

    // Create a simple tree
    let obj = vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
    let config = Config::default();

    // Save as .json.gz
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");
    save_json_file(&gz_path, &tree, &config).unwrap();

    // Decompress and verify
    let file = fs::File::open(&gz_path).unwrap();
    let mut decoder = GzDecoder::new(file);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&decompressed).unwrap();
    assert_eq!(parsed["name"], "Alice");
}

#[test]
fn test_save_jsonl_as_gzipped() {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tempfile::NamedTempFile;

    // Create JSONL tree
    let lines = vec![
        JsonNode::new(JsonValue::Object(vec![(
            "id".to_string(),
            JsonNode::new(JsonValue::Number(1.0)),
        )])),
        JsonNode::new(JsonValue::Object(vec![(
            "id".to_string(),
            JsonNode::new(JsonValue::Number(2.0)),
        )])),
    ];
    let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));
    let config = Config::default();

    // Save as .jsonl.gz
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("jsonl.gz");
    save_json_file(&gz_path, &tree, &config).unwrap();

    // Decompress and verify
    let file = fs::File::open(&gz_path).unwrap();
    let mut decoder = GzDecoder::new(file);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).unwrap();

    // Should be two lines
    let lines: Vec<&str> = decompressed.lines().collect();
    assert_eq!(lines.len(), 2);
}
```

**Step 2: Run tests**

Run: `cargo test test_save_json_as_gzipped test_save_jsonl_as_gzipped`
Expected: PASS

**Step 3: Commit**

```bash
git add src/file/saver.rs
git commit -m "test: add gzipped save functionality tests"
```

---

## Task 11: Add Format Switching Tests

**Files:**
- Test: `src/file/saver.rs` (test module)

**Step 1: Add format switching tests**

Add to test module in `src/file/saver.rs`:

```rust
#[test]
fn test_format_switching_json_to_gz() {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tempfile::NamedTempFile;

    // Create uncompressed JSON file
    let json_content = r#"{"test": "value"}"#;
    let temp_file = NamedTempFile::new().unwrap();
    let json_path = temp_file.path().with_extension("json");
    fs::write(&json_path, json_content).unwrap();

    // Load and save as .gz
    let tree = crate::file::loader::load_json_file(&json_path).unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");
    let config = Config::default();
    save_json_file(&gz_path, &tree, &config).unwrap();

    // Verify it's compressed
    let file = fs::File::open(&gz_path).unwrap();
    let mut decoder = GzDecoder::new(file);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).unwrap();
    assert!(decompressed.contains("test"));
}

#[test]
fn test_format_switching_gz_to_json() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create compressed JSON file
    let json_content = r#"{"test": "value"}"#;
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");
    let file = fs::File::create(&gz_path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(json_content.as_bytes()).unwrap();
    encoder.finish().unwrap();

    // Load and save as uncompressed
    let tree = crate::file::loader::load_json_file(&gz_path).unwrap();
    let json_path = temp_file.path().with_extension("json");
    let config = Config::default();
    save_json_file(&json_path, &tree, &config).unwrap();

    // Verify it's uncompressed
    let written = fs::read_to_string(&json_path).unwrap();
    assert!(written.contains("test"));
}
```

**Step 2: Run tests**

Run: `cargo test test_format_switching`
Expected: PASS

**Step 3: Commit**

```bash
git add src/file/saver.rs
git commit -m "test: add format switching tests (json ↔ gz)"
```

---

## Task 12: Add Backup Preservation Test

**Files:**
- Test: `src/file/saver.rs` (test module)

**Step 1: Add backup test**

Add to test module in `src/file/saver.rs`:

```rust
#[test]
fn test_backup_preserves_compression() {
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    // Create initial compressed file
    let json_content = r#"{"version": 1}"#;
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");
    let file = fs::File::create(&gz_path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(json_content.as_bytes()).unwrap();
    encoder.finish().unwrap();

    // Load and modify
    let mut tree = crate::file::loader::load_json_file(&gz_path).unwrap();
    if let JsonValue::Object(ref mut entries) = tree.root_mut().value_mut() {
        *entries[0].1.value_mut() = JsonValue::Number(2.0);
    }

    // Save with backup enabled
    let config = Config {
        create_backup: true,
        ..Default::default()
    };
    save_json_file(&gz_path, &tree, &config).unwrap();

    // Verify backup exists and is compressed
    let backup_path = gz_path.with_extension("json.gz.bak");
    assert!(backup_path.exists());

    // Decompress backup and verify original content
    let file = fs::File::open(&backup_path).unwrap();
    let mut decoder = GzDecoder::new(file);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).unwrap();
    assert!(decompressed.contains("\"version\": 1"));
}
```

**Step 2: Run test**

Run: `cargo test test_backup_preserves_compression`
Expected: PASS

**Step 3: Commit**

```bash
git add src/file/saver.rs
git commit -m "test: verify backup preserves compression state"
```

---

## Task 13: Create Integration Tests File

**Files:**
- Create: `tests/gzip_integration.rs`

**Step 1: Create integration test file**

Create `tests/gzip_integration.rs`:

```rust
//! Integration tests for gzip compression support.
//!
//! Tests roundtrip behavior, stdin handling, and large files.

use jsonquill::config::Config;
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::file::loader::load_json_file;
use jsonquill::file::saver::save_json_file;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_roundtrip_compressed_json() {
    // Create JSON tree
    let obj = vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
        (
            "active".to_string(),
            JsonNode::new(JsonValue::Boolean(true)),
        ),
    ];
    let original_tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
    let config = Config::default();

    // Save as .json.gz
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");
    save_json_file(&gz_path, &original_tree, &config).unwrap();

    // Load back
    let loaded_tree = load_json_file(&gz_path).unwrap();

    // Verify structure matches
    if let JsonValue::Object(original_entries) = original_tree.root().value() {
        if let JsonValue::Object(loaded_entries) = loaded_tree.root().value() {
            assert_eq!(original_entries.len(), loaded_entries.len());
            assert_eq!(original_entries[0].0, loaded_entries[0].0);
        } else {
            panic!("Loaded tree is not an object");
        }
    } else {
        panic!("Original tree is not an object");
    }
}

#[test]
fn test_roundtrip_compressed_jsonl() {
    // Create JSONL tree
    let lines = vec![
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
        ])),
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(2.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Bob".to_string())),
            ),
        ])),
    ];
    let original_tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));
    let config = Config::default();

    // Save as .jsonl.gz
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("jsonl.gz");
    save_json_file(&gz_path, &original_tree, &config).unwrap();

    // Load back
    let loaded_tree = load_json_file(&gz_path).unwrap();

    // Verify it's JSONL format with correct line count
    if let JsonValue::JsonlRoot(loaded_lines) = loaded_tree.root().value() {
        assert_eq!(loaded_lines.len(), 2);
    } else {
        panic!("Loaded tree is not JsonlRoot");
    }
}

#[test]
fn test_large_compressed_file() {
    // Generate large JSON programmatically (not stored in git)
    let mut large_array = Vec::new();
    for i in 0..100_000 {
        large_array.push(JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(i as f64))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String(format!("item_{}", i))),
            ),
        ])));
    }
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(large_array)));
    let config = Config::default();

    // Save as .json.gz
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");
    save_json_file(&gz_path, &tree, &config).unwrap();

    // Verify file is actually compressed (smaller than uncompressed would be)
    let compressed_size = fs::metadata(&gz_path).unwrap().len();
    assert!(
        compressed_size < 1_000_000,
        "Compressed file should be well under 1MB, got {} bytes",
        compressed_size
    );

    // Load and verify structure
    let loaded = load_json_file(&gz_path).unwrap();
    if let JsonValue::Array(elements) = loaded.root().value() {
        assert_eq!(elements.len(), 100_000);
    } else {
        panic!("Expected array");
    }

    // Cleanup happens automatically when temp_file drops
}
```

**Step 2: Run integration tests**

Run: `cargo test --test gzip_integration`
Expected: All 3 tests PASS

**Step 3: Commit**

```bash
git add tests/gzip_integration.rs
git commit -m "test: add gzip integration tests"
```

---

## Task 14: Run Full Test Suite

**Files:**
- None (verification step)

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests PASS

**Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 3: Run fmt check**

Run: `cargo fmt --check`
Expected: No formatting issues

**Step 4: If any issues, fix and commit**

If fmt or clippy report issues:
```bash
cargo fmt
cargo clippy --fix --allow-dirty
git add -A
git commit -m "chore: fix formatting and clippy warnings"
```

---

## Task 15: Update CLAUDE.md Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add gzip support to Usage section**

Find the "Usage" section and add after the stdin piping examples:

```markdown
# Gzip Compressed Files

jsonquill supports transparent gzip compression for `.json.gz` and `.jsonl.gz` files:

```bash
# Open a gzipped JSON file
./target/release/jsonquill config.json.gz

# Edit and save - remains compressed
# (in editor: make changes, :w)

# Save to different format
# :w config.json         - saves uncompressed
# :w config.json.gz      - saves compressed
# :w config.jsonl.gz     - saves as compressed JSONL

# Open gzipped JSONL
./target/release/jsonquill logs.jsonl.gz

# Pipe gzipped JSON from stdin (auto-detected by magic bytes)
gunzip -c data.json.gz | ./target/release/jsonquill
curl -s https://api.example.com/data.json.gz | ./target/release/jsonquill
```

**Compression details:**
- Extension-based: `.json.gz` and `.jsonl.gz` trigger compression/decompression
- Transparent: Editor operates on uncompressed JSON in memory
- Format switching: `:w filename.ext` determines output format
- Auto-detection: Gzipped stdin detected by magic bytes (0x1f 0x8b)
- Atomic writes: Compression happens before atomic rename (safe)
- Backups: `.bak` files preserve compression state of original
```

**Step 2: Verify documentation renders correctly**

Run: `cat CLAUDE.md | grep -A 20 "Gzip Compressed Files"`
Expected: Shows the new section

**Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add gzip compression support to CLAUDE.md"
```

---

## Task 16: Update README.md Documentation

**Files:**
- Modify: `README.md`

**Step 1: Add to feature list**

Find the feature list in the Status section and add:

```markdown
- Gzip compression support (transparent `.json.gz` and `.jsonl.gz` handling)
```

**Step 2: Add to Basic Usage section**

Find the "Basic Usage" section and add after stdin piping examples:

```markdown
# Open gzipped files (automatic decompression)
jsonquill config.json.gz
jsonquill logs.jsonl.gz

# Pipe gzipped JSON from stdin (auto-detected)
gunzip -c data.json.gz | jsonquill
curl -s https://api.example.com/data.json.gz | jsonquill

# Format switching with :w command
# :w newfile.json      - save uncompressed
# :w newfile.json.gz   - save compressed
```

**Step 3: Verify README renders correctly**

Run: `cat README.md | grep -i gzip`
Expected: Shows the new mentions

**Step 4: Commit**

```bash
git add README.md
git commit -m "docs: add gzip compression support to README.md"
```

---

## Task 17: Final Verification and Summary

**Files:**
- None (final verification)

**Step 1: Run complete build and test**

Run:
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
```

Expected: All pass, builds successfully

**Step 2: Manual smoke test (optional but recommended)**

```bash
# Create test file
echo '{"test": "value", "number": 42}' | gzip > /tmp/test.json.gz

# Open in editor
./target/release/jsonquill /tmp/test.json.gz

# In editor: make a change, :w, :q
# Verify file is still compressed
gunzip -c /tmp/test.json.gz
```

**Step 3: Review commits**

Run: `git log --oneline`
Expected: See all commits from this implementation

**Step 4: Create summary commit**

```bash
git commit --allow-empty -m "feat: gzip compression support complete

Complete implementation of transparent gzip compression:
- Load .json.gz and .jsonl.gz files (automatic decompression)
- Save with optional compression based on filename extension
- Format switching via :w command
- Stdin gzip auto-detection by magic bytes
- Atomic writes preserved (compression before rename)
- Backup files preserve compression state
- Comprehensive test coverage (unit + integration)
- Documentation updated (CLAUDE.md, README.md)

All tests passing. Ready for review and merge.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Completion

The implementation is complete when:
- ✅ All tests pass (`cargo test`)
- ✅ No clippy warnings (`cargo clippy -- -D warnings`)
- ✅ Code formatted (`cargo fmt --check`)
- ✅ Documentation updated (CLAUDE.md, README.md)
- ✅ Manual smoke test successful (optional)

**Next steps:**
- Use `@superpowers:finishing-a-development-branch` to merge or create PR
- Clean up worktree if needed
