# Templator — Claude Code Project Guide

Fast, Jinja2-based project template engine written in Rust.

## Mandatory after every Rust code change

Run both before marking any task complete — CI enforces both:

```bash
cargo fmt                        # auto-formats all Rust source
cargo clippy -- -D warnings      # must pass with zero warnings/errors
```

`cargo fmt` also runs automatically via a PostToolUse hook configured in
`.claude/settings.json` whenever you use Edit or Write on any file.

## Tests

```bash
cargo test          # unit tests (per module) + integration tests
```

All tests live either in `#[cfg(test)]` blocks inside each source module or in
`tests/integration_tests.rs`. Always run the full suite after changes.

## Project layout

| File | Role |
|---|---|
| `src/config.rs` | `Config` + `PostProcessStep` serde structs |
| `src/engine.rs` | Template walking, `__var__` path substitution, Tera rendering |
| `src/post_process.rs` | Post-generation steps: `replace`, `run_command` |
| `src/main.rs` | CLI entry point (clap) |
| `tests/integration_tests.rs` | End-to-end tests against `examples/basic-template/` |

## Template rules — non-negotiable

| Location | Syntax | Scope |
|---|---|---|
| Filename / dir name | `__variable_name__` | All files and dirs |
| File content | `{{ variable }}` (Tera/Jinja2) | **Only** `.template` files |

Files **without** a `.template` suffix are copied byte-for-byte. Never introduce
content placeholders in non-template files.

## Security constraints

- Variable values containing `..` or absolute paths are rejected by the engine.
- `replace` post-processing paths are validated against path traversal and
  symlink escape via `canonicalize`.
- Do not weaken these checks without explicit discussion.
