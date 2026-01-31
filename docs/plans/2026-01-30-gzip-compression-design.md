# Gzip Compression Support Design

**Date:** 2026-01-30
**Status:** Approved
**Feature:** Add transparent gzip compression support for `.json.gz` and `.jsonl.gz` files

## Overview

Add transparent gzip compression/decompression to jsonquill's file I/O layer, allowing users to edit compressed JSON files without manual decompression. The implementation will be isolated to the file loading and saving modules, leaving the core editor logic unchanged.

## Goals

- Support reading and writing `.json.gz` and `.jsonl.gz` files
- Maintain all existing features (atomic writes, backups, format preservation)
- Enable format switching via `:w` command (e.g., `:w file.json` decompresses, `:w file.json.gz` compresses)
- Auto-detect gzipped stdin by magic bytes
- Provide clear error messages for corrupted gzip files

## Non-Goals

- Single `.gz` extension support (files must be `.json.gz` or `.jsonl.gz`)
- Streaming decompression (files are fully decompressed into memory)
- Compression level configuration (uses default compression)
- Other compression formats (bzip2, xz, etc.)

## Architecture

### Dependency

Add `flate2 = "1.0"` to `Cargo.toml`. This is the standard Rust gzip library with 40M+ downloads and active maintenance.

### Design Principles

1. **Transparent compression**: Editor sees uncompressed JSON; compression only happens at file I/O boundaries
2. **Extension-based detection**: `.json.gz` and `.jsonl.gz` trigger compression/decompression
3. **Format switching**: Target filename extension determines output format
4. **Preserve existing behavior**: Atomic writes, backups, and format preservation continue working unchanged

### Module Changes

**src/file/loader.rs:**
- Detect `.gz` extension before JSONL detection
- Decompress file content before parsing
- Auto-detect gzipped stdin by magic bytes (0x1f 0x8b)

**src/file/saver.rs:**
- Detect `.gz` extension on target path
- Compress serialized JSON before atomic write
- Update both `save_json_file` and `save_jsonl` functions

## Implementation Details

### File Loading (loader.rs)

**Extension Detection:**

```rust
pub fn load_json_file<P: AsRef<Path>>(path: P) -> Result<JsonTree> {
    let path_ref = path.as_ref();

    // Check if file is gzipped
    let is_gzipped = path_ref.extension()
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

**Helper Functions:**

```rust
/// Reads and decompresses a gzipped file
fn read_gzipped_file<P: AsRef<Path>>(path: P) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let file = fs::File::open(path).context("Failed to open gzipped file")?;
    let mut decoder = GzDecoder::new(file);
    let mut content = String::new();
    decoder.read_to_string(&mut content)
        .context("Failed to decompress gzipped file")?;
    Ok(content)
}

/// Determines if file is JSONL format based on filename
/// Checks for .jsonl or .ndjson before .gz extension
fn determine_jsonl_format<P: AsRef<Path>>(path: P) -> bool {
    let path_str = path.as_ref().to_string_lossy();

    // Remove .gz if present
    let base = if path_str.ends_with(".gz") {
        &path_str[..path_str.len() - 3]
    } else {
        &path_str
    };

    base.ends_with(".jsonl") || base.ends_with(".ndjson")
}
```

**Stdin Auto-Detection:**

```rust
pub fn load_json_from_stdin() -> Result<JsonTree> {
    use std::io::{self, Read};

    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)
        .context("Failed to read from stdin")?;

    // Check for gzip magic bytes
    let content = if buffer.starts_with(&[0x1f, 0x8b]) {
        decompress_gzip_bytes(&buffer)?
    } else {
        String::from_utf8(buffer).context("Invalid UTF-8 in stdin")?
    };

    // Try JSON first, then JSONL
    parse_json(&content)
        .or_else(|_| parse_jsonl_content(&content))
        .context("Failed to parse from stdin")
}

fn decompress_gzip_bytes(bytes: &[u8]) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(bytes);
    let mut content = String::new();
    decoder.read_to_string(&mut content)
        .context("Failed to decompress gzipped stdin")?;
    Ok(content)
}
```

### File Saving (saver.rs)

**Main Save Function:**

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

    // Serialize JSON (existing logic unchanged)
    let mut json_str = if let Some(original) = tree.original_source() {
        serialize_preserving_format(tree.root(), original, config, 0)
    } else {
        serialize_node(tree.root(), config.indent_size, 0)
    };

    // Preserve trailing newline
    if let Some(original) = tree.original_source() {
        if original.ends_with('\n') && !json_str.ends_with('\n') {
            json_str.push('\n');
        }
    }

    // Validate before writing
    serde_json::from_str::<serde_json::Value>(&json_str)
        .context("Generated invalid JSON")?;

    // Write (compressed or uncompressed)
    write_file_atomic(path, json_str.as_bytes(), should_compress)?;

    Ok(())
}
```

**Atomic Write with Compression:**

```rust
/// Writes data to a file atomically, optionally compressing with gzip
fn write_file_atomic<P: AsRef<Path>>(
    path: P,
    data: &[u8],
    compress: bool
) -> Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    let path = path.as_ref();
    let temp_path = path.with_extension("tmp");

    if compress {
        // Write compressed
        let file = fs::File::create(&temp_path)
            .context("Failed to create temp file")?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(data)
            .context("Failed to write compressed data")?;
        encoder.finish()
            .context("Failed to finish compression")?;
    } else {
        // Write uncompressed
        fs::write(&temp_path, data)
            .context("Failed to write temp file")?;
    }

    // Atomic rename
    fs::rename(&temp_path, path)
        .context("Failed to rename temp file")?;

    Ok(())
}
```

**JSONL Saving:**

```rust
fn save_jsonl<P: AsRef<Path>>(
    path: P,
    tree: &JsonTree,
    config: &Config,
    compress: bool
) -> Result<()> {
    let path = path.as_ref();

    // Create backup if needed
    if config.create_backup && path.exists() {
        create_backup(path)?;
    }

    // Serialize JSONL (existing logic)
    let mut output = String::new();
    if let JsonValue::JsonlRoot(lines) = tree.root().value() {
        for (i, node) in lines.iter().enumerate() {
            let line = serialize_node_compact(node);
            serde_json::from_str::<serde_json::Value>(&line)
                .with_context(|| format!("Invalid JSON at line {}", i + 1))?;
            output.push_str(&line);
            output.push('\n');
        }
    }

    // Write atomically with optional compression
    write_file_atomic(path, output.as_bytes(), compress)?;

    Ok(())
}
```

## Error Handling

### Corrupted Gzip Files

`GzDecoder::read_to_string` returns an error for corrupted data. Propagate with context:
```rust
.context("Failed to decompress gzipped file - file may be corrupted")?
```

### Non-UTF8 Content

Handle when decompressed bytes aren't valid UTF-8:
```rust
decoder.read_to_string(&mut content)
    .context("Decompressed content is not valid UTF-8")?
```

### Disk Full During Compression

`GzEncoder::finish()` returns error if write fails. Temp file cleanup happens automatically (file descriptor dropped), and original file remains untouched due to atomic write pattern.

### Format Switching

- `:w data.json` from `data.json.gz` → decompress and save
- `:w data.json.gz` from `data.json` → compress and save
- `:w data.jsonl.gz` from `data.json` → compress and save as JSONL (user intent clear from extension)

### Backup Files

When `create_backup = true`:
- `file.json.gz` creates `file.json.gz.bak` (compressed backup)
- Simple `fs::copy` works because we copy the original file as-is
- Backup preserves compression state of original

## Testing Strategy

### Unit Tests (loader.rs)

```rust
#[test]
fn test_determine_jsonl_format() {
    assert!(determine_jsonl_format("data.jsonl.gz"));
    assert!(determine_jsonl_format("data.ndjson.gz"));
    assert!(!determine_jsonl_format("data.json.gz"));
    assert!(determine_jsonl_format("data.jsonl"));
}

#[test]
fn test_read_gzipped_json_file() {
    // Create temp file with compressed JSON
    // Read and verify decompression works
}

#[test]
fn test_read_gzipped_jsonl_file() {
    // Test .jsonl.gz files decompress and parse correctly
}

#[test]
fn test_corrupted_gzip_returns_error() {
    // Create file with .gz extension but invalid gzip data
    // Verify error message is helpful
}
```

### Unit Tests (saver.rs)

```rust
#[test]
fn test_save_json_as_gzipped() {
    // Save tree to .json.gz
    // Decompress and verify content matches
}

#[test]
fn test_save_jsonl_as_gzipped() {
    // Save JSONL tree to .jsonl.gz
    // Verify each line decompresses correctly
}

#[test]
fn test_format_switching_json_to_gz() {
    // Load .json, save as .json.gz
    // Verify compression happened
}

#[test]
fn test_format_switching_gz_to_json() {
    // Load .json.gz, save as .json
    // Verify decompression happened
}

#[test]
fn test_atomic_write_with_compression() {
    // Verify temp file is created and renamed
    // Verify original file not touched until rename
}

#[test]
fn test_backup_preserves_compression() {
    // Enable create_backup
    // Save .json.gz file twice
    // Verify .json.gz.bak is also compressed
}
```

### Integration Tests (tests/gzip_integration.rs)

```rust
#[test]
fn test_roundtrip_compressed_json() {
    // Create JSON → save as .json.gz → load → verify identical
}

#[test]
fn test_stdin_gzipped_json() {
    // Pipe compressed JSON to stdin
    // Verify auto-detection works
}

#[test]
fn test_large_compressed_file() {
    use tempfile::NamedTempFile;

    // Generate large JSON programmatically (not stored in git)
    let mut large_array = Vec::new();
    for i in 0..100_000 {
        large_array.push(JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(i as f64))),
            ("name".to_string(), JsonNode::new(JsonValue::String(format!("item_{}", i)))),
        ])));
    }
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(large_array)));

    // Save as .json.gz
    let temp_file = NamedTempFile::new().unwrap();
    let gz_path = temp_file.path().with_extension("json.gz");
    save_json_file(&gz_path, &tree, &Config::default()).unwrap();

    // Verify file is actually compressed (smaller than uncompressed)
    let compressed_size = fs::metadata(&gz_path).unwrap().len();
    assert!(compressed_size < 1_000_000); // Should be well under 1MB compressed

    // Load and verify structure matches
    let loaded = load_json_file(&gz_path).unwrap();
    // Verify structure...

    // Cleanup happens automatically when temp_file drops
}
```

### Manual Testing Checklist

- [ ] Open `.json.gz` file in editor
- [ ] Edit values and save
- [ ] `:w newfile.json` to decompress
- [ ] `:w newfile.json.gz` to re-compress
- [ ] Pipe gzipped JSON via stdin: `gunzip -c file.json.gz | jsonquill`
- [ ] Open `.jsonl.gz` file
- [ ] Verify backup creates `.json.gz.bak`
- [ ] Test with large files (>10MB compressed, generated during test)

## Implementation Steps

1. Add `flate2` dependency to `Cargo.toml`
2. Implement helper functions in `loader.rs`
3. Modify `load_json_file` and `load_json_from_stdin`
4. Implement `write_file_atomic` in `saver.rs`
5. Modify `save_json_file` and `save_jsonl`
6. Add unit tests to `loader.rs` and `saver.rs`
7. Create `tests/gzip_integration.rs` with integration tests
8. Manual testing with real `.gz` files
9. Update documentation (CLAUDE.md, README.md)

## Documentation Updates

**CLAUDE.md:**
- Add `.gz` support to "Usage" section
- Document `.json.gz` and `.jsonl.gz` extensions
- Add example: `jsonquill data.json.gz`

**README.md:**
- Add gzip support to feature list
- Add examples in "Basic Usage" section
- Note compression is transparent to the user

## Risks & Mitigations

**Risk:** Large decompressed files consume excessive memory
**Mitigation:** Document memory requirements; future enhancement could add streaming

**Risk:** Corrupted gzip files cause poor error messages
**Mitigation:** Add clear context to all decompression errors

**Risk:** Users expect compression level configuration
**Mitigation:** Use sensible default (level 6); can add config option later if requested

## Future Enhancements

- Compression level configuration (`Config.compression_level`)
- Other compression formats (bzip2, xz, zstd)
- Streaming decompression for extremely large files
- Progress indicator for large file decompression
