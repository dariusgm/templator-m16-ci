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
| `examples/README.md` | Guide to the complete example and reference documentation for all templator features. |

### `complete-example` — All Features

Demonstrates **all** templator features (variable substitution, Tera rendering, static files, `exclude`, `pre_processing`, `replace`, `preserve_sections`, and git hooks) in a single Python project template.

| File | Purpose |
|---|---|
| `examples/complete-example/templator.json` | Config with all feature types: top-level `exclude`, `pre_processing` exclude, `replace` post-processing, and `preserve_sections` post-processing. |
| `examples/complete-example/template/` | Python project template demonstrating all features. |
| `examples/complete-example/template/pyproject.toml.template` | Tera template with `__COPYRIGHT_YEAR__` literal placeholder for `replace` post-processing, and `preserve_sections` markers for user-edited dependencies. |
| `examples/complete-example/template/README.md.template` | Tera rendering with full feature documentation. |
| `examples/complete-example/template/src/__project_name__/main.py.template` | Demonstrates filename variable substitution (`__project_name__` → project name). |
| `examples/complete-example/template/.gitignore` | Static file copied verbatim; not rendered. |
| `examples/complete-example/template/docs/guide.md` | Public docs included in output. |
| `examples/complete-example/template/docs/private/notes.md` | Private docs excluded by top-level `exclude` pattern `docs/private/`. |
| `examples/complete-example/template/.git/hooks/pre-commit` | Executable git hook; `.git/` directory is partially included via selective exclude patterns. |
| `examples/complete-example/template/main.py.draft` | Draft file excluded by top-level `exclude` pattern `*.draft`. |
| `examples/complete-example/template/temp/scratch.txt` | Temporary file excluded by `pre_processing` exclude pattern `temp/`. |
| `examples/complete-example/template/debug.log` | Log file excluded by `pre_processing` exclude pattern `*.log`. |
| `examples/complete-example-answers.json` | Example answers file with project name, Python version, and author. |