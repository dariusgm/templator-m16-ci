# Project File Reference

## Root Configuration & Documentation

| File | Purpose |
|---|---|
| `README.md` | User-facing documentation explaining how to install, use, and understand templator—the Jinja2-based template engine written in Rust. |
| `CLAUDE.md` | Developer instructions for Claude Code: mandatory `cargo fmt` and `cargo clippy` runs after Rust changes, `cargo test` for unit/integration tests, and `cargo-tarpaulin` for code coverage (95% minimum threshold). |
| `LICENSE` | MIT license granting permission to use, modify, and distribute the software with attribution. |
| `Cargo.toml` | Rust project manifest declaring dependencies (serde, tera, clap, walkdir), build targets, and release optimizations (LTO, stripping). |
| `Cargo.lock` | Locked dependency versions ensuring reproducible builds across environments. |
| `.gitignore` | Git ignore patterns for Rust build artifacts (`target/`, `debug/`, `*.pdb`) and test output (`mutants.out*/`). |

## Source Code

| File | Purpose |
|---|---|
| `src/main.rs` | CLI entry point using clap to parse `--template`, `--config`, and `--output` arguments; orchestrates template generation and post-processing. |
| `src/lib.rs` | Library root that exports the public API: `Config`, `TemplateEngine`, and `PostProcessor`. |
| `src/config.rs` | Serde structs for deserializing `templator.json`: `Config` (name, variables, exclude, pre_processing, post_processing), `PreProcessStep` enum (Exclude), and `PostProcessStep` enum (Replace, PreserveSections). |
| `src/engine.rs` | Core template processing: walks the template directory, applies `__var__` substitution to filenames, renders `.template` files via Tera with variable context, and preserves source file permissions (including Unix executable bits) on output files. |
| `src/pre_process.rs` | Pre-generation phase: `PreProcessor::effective_exclude` merges `config.exclude` with patterns from `pre_processing` Exclude steps to produce the list fed to the engine's path filter. |
| `src/post_process.rs` | Post-generation steps: `replace` for literal string substitution and `preserve_sections` for section preservation, with path traversal and symlink escape protection. |

## Tests

| File | Purpose |
|---|---|
| `tests/integration_tests.rs` | End-to-end tests validating the full pipeline: template rendering, file/directory copying, variable substitution, and post-processing against `examples/basic-template/`. |

## CI/CD & Configuration

| File | Purpose |
|---|---|
| `.claude/settings.json` | Claude Code harness configuration enabling a `PostToolUse` hook that runs `cargo fmt` after every Edit or Write action on Rust files. |
| `.github/workflows/ci.yml` | GitHub Actions workflow that tests, lints, and builds binaries for Linux x86_64/ARM64, Windows x86_64, and macOS Apple Silicon; uploads artifacts on success. |

## Examples

| File | Purpose |
|---|---|
| `examples/README.md` | Comprehensive guide to all examples with instructions for running each one and explanation of post-processing types. |

### `basic-template` — Replace Post-Processing

Demonstrates the `replace` post-processing step for literal string substitution.

| File | Purpose |
|---|---|
| `examples/basic-template/templator.json` | Config with `replace` step that substitutes `__AUTHOR__` with "Darius" in `pyproject.toml`. |
| `examples/basic-template/template/` | Python project template with variable substitution and static files. |
| `examples/basic-template/template/pyproject.toml.template` | Tera template for Python metadata; author placeholder replaced via post-processing. |
| `examples/basic-template/template/src/__project_name__/main.py.template` | Nested template file demonstrating `__var__` directory renaming and Tera variable injection. |
| `examples/basic-template/template/.gitignore` | Static file copied verbatim; not rendered. |

### `full-featured-example` — Multiple Post-Processing Steps

Demonstrates combining multiple post-processing steps in a single template.

| File | Purpose |
|---|---|
| `examples/full-featured-example/templator.json` | Config with two `replace` steps (version, author, copyright year). |
| `examples/full-featured-example/template/Cargo.toml.template` | Rust manifest template with version and author placeholders for post-processing substitution. |
| `examples/full-featured-example/template/src/lib/mod.rs.template` | Rust library module with copyright year placeholder for post-processing substitution. |
| `examples/full-featured-example/template/README.md.template` | Documentation template with project name and description variables. |
| `examples/full-featured-example/template/build.sh` | Shell script template for the Rust project. |

### `exclude-example` — File Exclusion

Demonstrates the `exclude` feature to prevent certain files from being copied to output.

| File | Purpose |
|---|---|
| `examples/exclude-example/templator.json` | Config with `exclude` patterns for drafts, private docs, build artifacts, and temp files. |
| `examples/exclude-example/template/README.md.template` | Template documenting which files are included vs. excluded. |
| `examples/exclude-example/template/src/main.rs` | Source file that is **included** in output. |
| `examples/exclude-example/template/src/main.rs.draft` | Draft version matching `*.draft` pattern; **excluded** from output. |
| `examples/exclude-example/template/docs/guide.md` | Public documentation that is **included** in output. |
| `examples/exclude-example/template/docs/private/notes.md` | Private notes in `docs/private/` directory; **excluded** from output. |
| `examples/exclude-example/template/.env.example` | Example env file excluded by exact name match. |
| `examples/exclude-example/template/build/artifact.o` | Build output in `build/` directory; **excluded** from output. |
| `examples/exclude-example/template/temp/cache.tmp` | Temporary file matching `**/*.tmp` pattern; **excluded** from output. |

### `git-hooks-example` — Git Hooks with Selective `.git/` Inclusion

Demonstrates including `.git/hooks/` scripts while excluding the rest of the `.git/` directory using selective exclude patterns. Useful for templates that want to provision git hooks (e.g., pre-commit checks) in generated projects.

| File | Purpose |
|---|---|
| `examples/git-hooks-example/templator.json` | Config with exclude patterns that keep only `.git/hooks/` (no objects, refs, config, etc.). |
| `examples/git-hooks-example/template/.git/hooks/pre-commit` | Executable hook that runs `cargo fmt --check` before each commit. |
| `examples/git-hooks-example/template/.git/hooks/prepare-commit-msg` | Optional hook with example code for prepending issue numbers to commits. |
| `examples/git-hooks-example/template/README.md.template` | Documentation explaining the included hooks and how to customize them. |
| `examples/git-hooks-example/template/src/main.rs.template` | Sample Rust source file with author and project name placeholders. |
| `examples/git-hooks-example/template/.gitignore` | Standard Rust `.gitignore` file. |

### `pre-processing-example` — Pre-Processing Exclusion

Demonstrates using `pre_processing` Exclude steps alongside `config.exclude` to show that both mechanisms are additive.

| File | Purpose |
|---|---|
| `examples/pre-processing-example/templator.json` | Config with both `exclude` and `pre_processing` Exclude patterns. |
| `examples/pre-processing-example/template/README.md.template` | Template file included in output. |
| `examples/pre-processing-example/template/src/__project_name__/main.py.template` | Nested template demonstrating variable-based directory renaming. |
| `examples/pre-processing-example/template/docs/guide.md` | Static file included verbatim. |
| `examples/pre-processing-example/template/docs/internal/architecture.md` | Internal doc excluded by pre_processing Exclude (docs/internal/). |
| `examples/pre-processing-example/template/temp/scratch.txt` | Temp file excluded by pre_processing Exclude (temp/). |
| `examples/pre-processing-example/template/debug.log` | Log file excluded by pre_processing Exclude (*.log). |
| `examples/pre-processing-example/template/main.py.draft` | Draft file excluded by config.exclude (*.draft). |

### `preserve-sections-example` — Preserve Sections Across Regenerations

Demonstrates the `preserve_sections` post-processing step for protecting user-edited sections when templates are regenerated. Supports marking sections with begin/end markers so user content survives template updates.

| File | Purpose |
|---|---|
| `examples/preserve-sections-example/templator.json` | Config with `preserve_sections` step targeting `pyproject.toml` with `# [preserve:begin]` and `# [preserve:end]` markers. |
| `examples/preserve-sections-example/template/pyproject.toml.template` | Python project metadata template with user-controlled section marked between begin/end markers for custom dependencies or settings. |
| `examples/preserve-sections-example-answers.json` | Example answers file with project name, Python version, and author. |