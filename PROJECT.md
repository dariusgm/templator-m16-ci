# Project File Reference

## Root Configuration & Documentation

| File | Purpose |
|---|---|
| `README.md` | User-facing documentation explaining how to install, use, and understand templator—the Jinja2-based template engine written in Rust. |
| `CLAUDE.md` | Developer instructions for Claude Code, specifying mandatory `cargo fmt` and `cargo clippy` runs after Rust changes. |
| `LICENSE` | MIT license granting permission to use, modify, and distribute the software with attribution. |
| `Cargo.toml` | Rust project manifest declaring dependencies (serde, tera, clap, walkdir), build targets, and release optimizations (LTO, stripping). |
| `Cargo.lock` | Locked dependency versions ensuring reproducible builds across environments. |
| `.gitignore` | Git ignore patterns for Rust build artifacts (`target/`, `debug/`, `*.pdb`) and test output (`mutants.out*/`). |

## Source Code

| File | Purpose |
|---|---|
| `src/main.rs` | CLI entry point using clap to parse `--template`, `--config`, and `--output` arguments; orchestrates template generation and post-processing. |
| `src/lib.rs` | Library root that exports the public API: `Config`, `TemplateEngine`, and `PostProcessor`. |
| `src/config.rs` | Serde structs for deserializing `templator.json`: `Config` (name, variables, post_processing) and `PostProcessStep` enum (Replace, RunCommand, PreserveSections). |
| `src/engine.rs` | Core template processing: walks the template directory, applies `__var__` substitution to filenames, and renders `.template` files via Tera with variable context. |
| `src/post_process.rs` | Post-generation steps: `replace` for literal string substitution and `run_command` to execute shell commands, with path traversal and symlink escape protection. |

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
| `examples/basic-template/templator.json` | Example configuration file declaring project name, variables (e.g., `project_name`, `python_version`), and post-processing steps. |
| `examples/basic-template/template/` | Example template directory containing `.template` files (Tera-rendered) and static files; demonstrates `__project_name__` directory renaming and variable substitution. |
| `examples/basic-template/template/pyproject.toml.template` | Tera template for Python project metadata; content rendered with variables from `templator.json`. |
| `examples/basic-template/template/src/__project_name__/main.py.template` | Nested template file showing both directory-level `__var__` substitution and file-level Tera templating. |
| `examples/basic-template/template/.gitignore` | Static file copied verbatim without rendering; no `.template` suffix means content is never modified. |