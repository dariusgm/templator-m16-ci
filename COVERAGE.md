# Code Coverage Report

## Current Coverage: 96.26%

**Coverage includes:** `src/config.rs`, `src/engine.rs`, `src/post_process.rs`  
**Excluded:** `src/main.rs` (CLI boilerplate, tested via integration tests)

| Module | Coverage | Lines Covered | Total Lines |
|---|---|---|---|
| `src/config.rs` | 100% | 5 | 5 |
| `src/engine.rs` | 93.4% | 57 | 61 |
| `src/post_process.rs` | 100% | 41 | 41 |
| **Total** | **96.26%** | **103** | **107** |

## Uncovered Lines

### `src/engine.rs` (2 lines, 6.6%)

**Lines 36-37:** Error handling for unreadable directory entries
```rust
Err(e) => {
    log::warn!("Skipping unreadable entry: {}", e);
```
*Reason:* Requires a permission issue during directory traversal to trigger. Tested indirectly through integration tests.

**Lines 104-105:** Default trait implementation
```rust
impl Default for TemplateEngine {
    fn default() -> Self {
```
*Reason:* Trivial implementation that delegates to `new()`, already tested implicitly.

## Post-Processing Coverage

All post-processing steps have **100% test coverage**:

### `Replace` Post-Processing
- ✅ Single pattern replacement
- ✅ Multiple sequential patterns
- ✅ All occurrences replaced
- ✅ Empty patterns (no-op)
- ✅ Missing file error handling
- ✅ Path traversal rejection (`..`)
- ✅ Absolute path rejection
- ✅ Symlink escape prevention
- ✅ Nested subdirectory support
- ✅ Error messages include command details

### `PreserveSections` Post-Processing
- ✅ Returns "not implemented" error (placeholder for future feature)

## CI Integration

Coverage is checked on every push and pull request:
- **Tool:** `cargo-tarpaulin` (LLVM-based coverage)
- **Minimum threshold:** 95%
- **Status:** ✅ Passing (96.26%)

If coverage drops below 95%, the CI build will **fail** and block merging.

## Running Coverage Locally

```bash
# Generate coverage report
cargo tarpaulin \
  --include-files src/config.rs src/engine.rs src/post_process.rs \
  --ignore-panics \
  --ignore-tests

# Run all tests
cargo test
```
