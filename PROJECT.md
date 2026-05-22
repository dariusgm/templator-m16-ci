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

### `run-command-example` — RunCommand Post-Processing

Demonstrates the `run_command` post-processing step for executing shell commands (e.g., chmod).

| File | Purpose |
|---|---|
| `examples/run-command-example/templator.json` | Config with two `run_command` steps that make generated shell scripts executable via `chmod +x`. |
| `examples/run-command-example/template/README.md.template` | Documentation template explaining how to use the generated scripts. |
| `examples/run-command-example/template/scripts/setup.sh` | Shell script template; becomes executable after generation via post-processing. |
| `examples/run-command-example/template/scripts/deploy.sh` | Shell script template; becomes executable after generation via post-processing. |

### `full-featured-example` — Multiple Post-Processing Steps

Demonstrates combining multiple post-processing steps (`replace` and `run_command`) in a single template.

| File | Purpose |
|---|---|
| `examples/full-featured-example/templator.json` | Config with two `replace` steps (version, author, copyright year) and one `run_command` step (chmod). |
| `examples/full-featured-example/template/Cargo.toml.template` | Rust manifest template with version and author placeholders for post-processing substitution. |
| `examples/full-featured-example/template/src/lib/mod.rs.template` | Rust library module with copyright year placeholder for post-processing substitution. |
| `examples/full-featured-example/template/README.md.template` | Documentation template with project name and description variables. |
| `examples/full-featured-example/template/build.sh` | Shell script template; becomes executable via `run_command` post-processing. |

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