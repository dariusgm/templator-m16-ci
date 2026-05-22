# Templator Examples

This directory contains example templates demonstrating templator's features and post-processing capabilities.

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
  --template examples/basic-template/template \
  --config examples/basic-template/templator.json \
  --output /tmp/my-python-project
```

**What happens:**
1. Files are rendered (`.template` suffix stripped)
2. Directories are renamed (`__project_name__` → `my_project`)
3. Post-processing replaces `__AUTHOR__` with `Darius` in `pyproject.toml`

---

### 2. `run-command-example` — RunCommand Post-Processing

Demonstrates the `run_command` post-processing step for executing shell commands.

**Features:**
- Project with executable shell scripts
- Multiple `run_command` steps to set script permissions
- Scripts generated with content templating

**Run:**
```bash
templator \
  --template examples/run-command-example/template \
  --config examples/run-command-example/templator.json \
  --output /tmp/my-scripts-project
```

**What happens:**
1. Template files are rendered
2. Post-processing runs `chmod +x` on generated scripts
3. Scripts become immediately executable without manual permission changes

**Use case:** Ensure shell scripts, Python entry points, or compiled binaries have correct permissions after generation.

---

### 3. `full-featured-example` — Multiple Post-Processing Steps

Demonstrates combining multiple post-processing types in a single template.

**Features:**
- Rust library template with `Cargo.toml`
- Multiple variable substitutions (version, author, year)
- Executable script setup via `run_command`
- Two-step `replace` on separate files

**Run:**
```bash
templator \
  --template examples/full-featured-example/template \
  --config examples/full-featured-example/templator.json \
  --output /tmp/my-rust-lib
```

**What happens:**
1. Tera renders all `.template` files with variables
2. First `replace` step substitutes version and author in `Cargo.toml`
3. Second `replace` step substitutes copyright year in `src/lib/mod.rs`
4. `run_command` makes `build.sh` executable

**Use case:** Complex templates requiring both content templating and post-generation setup.

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

### `run_command` — Execute Shell Commands

```json
{
  "type": "run_command",
  "command": "chmod +x scripts/setup.sh"
}
```

Command runs via `sh -c` with the output directory as the working directory.

**Constraints:**
- Command must exit with status 0 (non-zero exit aborts generation)
- Works within the generated output directory only

**Use cases:**
- Set executable permissions on scripts
- Run build/compilation steps
- Initialize repositories or databases
- Format generated code

### `preserve_sections` — Reserved (Not Yet Implemented)

This post-processing type is planned for future releases. It will allow preserving user-edited sections of generated files across template regenerations.

---

## Testing Examples

To verify all examples work correctly:

```bash
# Test basic-template
cargo run -- \
  --template examples/basic-template/template \
  --config examples/basic-template/templator.json \
  --output /tmp/test-basic

# Test run-command-example
cargo run -- \
  --template examples/run-command-example/template \
  --config examples/run-command-example/templator.json \
  --output /tmp/test-run-command

# Test full-featured-example
cargo run -- \
  --template examples/full-featured-example/template \
  --config examples/full-featured-example/templator.json \
  --output /tmp/test-full-featured
```

Each example exercises different aspects of templator's functionality and can serve as a reference for your own templates.
