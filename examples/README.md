# Templator Examples

This directory contains example templates demonstrating templator's features.

## Running Examples

All examples use the same CLI pattern:

```bash
templator \
  --template examples/<example-name> \
  --answers examples/<example-name>-answers.json \
  --output /tmp/<output-dir>
```

## Examples by Post-Processing Type

### 1. `basic-template` — Replace Post-Processing

Demonstrates the `replace` post-processing step for literal string substitution.

**Features:**
- Python project template with `pyproject.toml`
- Variable substitution in filenames (`__project_name__`)
- Content templating with Tera/Jinja2 syntax
- Replace step to substitute author placeholder with actual name

**Run:**
```bash
templator \
  --template examples/basic-template \
  --answers examples/basic-answers.json \
  --output /tmp/my-python-project
```

**What happens:**
1. Files are rendered (`.template` suffix stripped)
2. Directories are renamed (`__project_name__` → `my_project`)
3. Post-processing replaces `__AUTHOR__` with `Darius` in `pyproject.toml`

---

### 2. `full-featured-example` — Multiple Post-Processing Steps

Demonstrates combining multiple post-processing types in a single template.

**Features:**
- Rust library template with `Cargo.toml`
- Multiple variable substitutions (version, author, year)
- Two-step `replace` on separate files

**Run:**
```bash
templator \
  --template examples/full-featured-example \
  --answers examples/basic-answers.json \
  --output /tmp/my-rust-lib
```

**What happens:**
1. Tera renders all `.template` files with variables
2. First `replace` step substitutes version and author in `Cargo.toml`
3. Second `replace` step substitutes copyright year in `src/lib/mod.rs`

**Use case:** Complex templates requiring multiple content replacements.

---

### 3. `exclude-example` — File Exclusion

Demonstrates the `exclude` feature to prevent certain files from being copied to the output directory.

**Features:**
- Pattern-based file exclusion (extension, directory, recursive)
- Multiple exclusion patterns in a single config
- Shows which files are included vs. excluded

**Run:**
```bash
templator \
  --template examples/exclude-example \
  --answers examples/basic-answers.json \
  --output /tmp/project-with-exclusions
```

**What happens:**
1. Files matching `*.draft` are excluded (draft versions)
2. `docs/private/` directory is excluded entirely
3. `.env.example` is excluded by exact name
4. `build/` directory is excluded (build artifacts)
5. All `**/*.tmp` files are excluded (temporary files)
6. Only public source files and documentation are included

**Exclude patterns:**
- `*.ext` — exclude files with specific extension
- `dir/` — exclude entire directory
- `filename` — exclude by exact name
- `**/*.ext` — exclude files with extension at any depth

**Use case:** Remove sensitive files, build artifacts, drafts, and temporary files from generated projects.

---

### 4. `git-hooks-example` — Git Hooks via Selective `.git/` Inclusion

Demonstrates how to include git hooks (`.git/hooks/`) while excluding the rest of the `.git/` directory using selective exclude patterns.

**Features:**
- Rust project template with pre-commit hook for code formatting
- Pre-commit hook runs `cargo fmt --check` on every commit
- Optional prepare-commit-msg hook (with example configuration)
- Uses exclude patterns to keep only `.git/hooks/` (no objects, refs, config)

**Run:**
```bash
templator \
  --template examples/git-hooks-example \
  --answers examples/git-hooks-example-answers.json \
  --output /tmp/rust-with-hooks
```

**What happens:**
1. Template files are rendered with project name and author
2. `.git/hooks/pre-commit` and `.git/hooks/prepare-commit-msg` are copied (executable)
3. All other `.git/` internals are excluded (objects/, refs/, config, HEAD, etc.)
4. Generated project has working git hooks ready to use

**Exclude patterns used:**
```json
"exclude": [
  ".git/objects/",
  ".git/refs/",
  ".git/logs/",
  ".git/config",
  ".git/HEAD",
  ".git/description"
]
```

This keeps only the hooks directory while excluding the git database.

**Use case:** Create templates that ship pre-configured development tooling (linters, formatters, code quality checks) via git hooks.

---

## Post-Processing Reference

### `replace` — String Substitution

```json
{
  "type": "replace",
  "file": "path/to/file.txt",
  "patterns": [
    { "from": "OLD_TEXT", "to": "NEW_TEXT" },
    { "from": "ANOTHER", "to": "REPLACEMENT" }
  ]
}
```

Patterns are applied sequentially. The output of one becomes the input of the next.

**Constraints:**
- File path must be relative (no `..` or absolute paths)
- Cannot escape the output directory via symlinks

**Use cases:**
- Substitute author/version/license information
- Replace configuration placeholders
- Update code comments or documentation

## Exclusion Reference

### `exclude` — File and Directory Exclusion

```json
{
  "exclude": [
    "*.draft",
    "docs/private/",
    ".env.example",
    "build/",
    "**/*.tmp"
  ]
}
```

Files and directories matching any pattern in the `exclude` array will not be copied to the output directory.

**Supported patterns:**
- `*.ext` — files with specific extension (e.g., `*.tmp`, `*.bak`)
- `filename` — exact filename match (e.g., `.env.example`)
- `dir/` — entire directory (trailing slash required)
- `**/*.ext` — files with extension at any directory depth

**Constraints:**
- Patterns are matched against relative paths from template root
- Directory patterns must end with `/`
- Paths use forward slashes regardless of OS

**Use cases:**
- Exclude sensitive example files (`.env.example`)
- Remove build artifacts (`build/`, `dist/`)
- Skip draft or temporary files (`*.draft`, `*.tmp`)
- Hide internal documentation (`docs/private/`)
- Remove version control (`build/`)

### `preserve_sections` — Reserved (Not Yet Implemented)

This post-processing type is planned for future releases. It will allow preserving user-edited sections of generated files across template regenerations.

---

## Testing Examples

To verify all examples work correctly:

```bash
# Test basic-template
cargo run -- \
  --template examples/basic-template \
  --answers examples/basic-answers.json \
  --output /tmp/test-basic

# Test full-featured-example
cargo run -- \
  --template examples/full-featured-example \
  --answers examples/basic-answers.json \
  --output /tmp/test-full-featured

# Test exclude-example
cargo run -- \
  --template examples/exclude-example \
  --answers examples/basic-answers.json \
  --output /tmp/test-exclude

# Test git-hooks-example
cargo run -- \
  --template examples/git-hooks-example \
  --answers examples/git-hooks-example-answers.json \
  --output /tmp/test-hooks

# Verify git hooks were copied and are executable
ls -la /tmp/test-hooks/.git/hooks/
```

Each example exercises different aspects of templator's functionality and can serve as a reference for your own templates.
