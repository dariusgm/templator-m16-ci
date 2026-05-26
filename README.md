# templator

Fast, Jinja2-based project template engine written in Rust. Generates a project directory from a template directory and a JSON config file in under 100 ms.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
templator \
  --template path/to/template \
  --answers  path/to/answers.json \
  --output   path/to/output

# Flags
#   --template   Template directory (must contain templator.json)
#   --answers    JSON file with variable values
#   --output     Output directory (created if it does not exist)
#   -v, --verbose    Enable debug logging
```

## How it works

templator walks the template directory and, for each entry:

| Entry | Rule |
|---|---|
| Directory | Created in the output tree; its name may contain `__var__` placeholders |
| File **with** `.template` suffix | Content rendered with Tera (Jinja2 syntax); `.template` stripped from the output filename |
| File **without** `.template` suffix | Copied byte-for-byte; content is never touched |

This makes template boundaries explicit: you can see at a glance which files are rendered and which are inert.

### Variable substitution

Two independent substitution layers exist, applied in separate passes:

| Layer | Syntax | Scope |
|---|---|---|
| Path names | `__variable_name__` | Every file and directory name |
| File content | `{{ variable }}` (Tera / Jinja2) | Only `.template` files |

The `.template` suffix is also stripped from intermediate directory names, so `views.template/index.html.template` becomes `views/index.html` in the output.

### Available in Tera templates

| Variable | Source |
|---|---|
| `{{ name }}` | `name` field in `templator.json` |
| `{{ description }}` | `description` field (optional) |
| `{{ <key> }}` | Any entry in `variables` |

Variables defined in `variables` override the top-level `name` and `description` fields if they share the same key.

Full Tera syntax is available: filters (`{{ value \| upper }}`), conditionals (`{% if … %}`), loops (`{% for … %}`), and the `default` filter (`{{ optional \| default(value="fallback") }}`).

### Pre-processing

`pre_processing` steps run before the engine walks the template directory. Currently, only the `exclude` step type is supported.

#### `exclude` — glob patterns

```json
"pre_processing": [
  {
    "type": "exclude",
    "patterns": ["*.log", "temp/", "docs/internal/"]
  }
]
```

| Field | Type | Required | Description |
|---|---|---|---|
| `type` | string | yes | Must be `"exclude"` |
| `patterns` | array of strings | yes | Glob patterns using the same syntax as `exclude` |


## Config file (`templator.json`)

```json
{
  "name": "my-python-project",
  "description": "A Python project template",
  "variables": {
    "project_name": "my_project",
    "python_version": "3.12",
    "python_version_short": "py312",
    "author": "Darius"
  },
  "pre_processing": [
    {
      "type": "exclude",
      "patterns": ["*.draft", ".env.example"]
    }
  ],
  "post_processing": [
    {
      "type": "replace",
      "file": "pyproject.toml",
      "patterns": [
        { "from": "__AUTHOR__", "to": "Darius" }
      ]
    }
  ]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Project name (available as `{{ name }}` in templates) |
| `description` | string | no | Available as `{{ description }}` in templates |
| `variables` | object | yes | Key/value pairs injected into the Tera context |
| `pre_processing` | array | no | Steps to run before file generation |
| `post_processing` | array | no | Steps to run after file generation |

## Processing Pipeline Reference

### Pre-Processing Phase

Pre-processing runs **before** the template engine walks the template directory. Currently supports file exclusion patterns.

#### `exclude` — Exclusion patterns

```json
"pre_processing": [
  {
    "type": "exclude",
    "patterns": ["*.log", "temp/", "docs/internal/"]
  }
]
```

Use to exclude files and directories from template generation. Multiple exclusion steps are additive (a file is excluded if it matches ANY pattern).

| Field | Type | Description |
|---|---|---|
| `type` | string | Must be `"exclude"` |
| `patterns` | array | Glob patterns using the same syntax documented in Exclusion Patterns |

---

### Post-Processing Phase

Post-processing runs **after** the template engine generates files. Supports literal string replacement and section preservation.

#### Summary Table

| Step Type | Purpose | File Modified | When to Use |
|---|---|---|---|
| `replace` | Find-and-replace literal strings | Yes | Substitute author/version/license info; update placeholders |
| `preserve_sections` | Protect user edits across regenerations | Yes | Keep custom config; preserve user-added dependencies |

### Post-processing steps

#### `replace` — literal string substitution in a generated file

```json
{
  "type": "replace",
  "file": "pyproject.toml",
  "patterns": [
    { "from": "__AUTHOR__", "to": "Darius" }
  ]
}
```

Patterns are applied sequentially: the output of one becomes the input of the next.  
Paths must be relative and must not contain `..`.

#### `preserve_sections` — protect user-edited sections across regenerations

```json
{
  "type": "preserve_sections",
  "file": "pyproject.toml",
  "begin_marker": "# [preserve:begin]",
  "end_marker": "# [preserve:end]"
}
```

Wraps user-controlled sections with begin and end markers. When the template is regenerated on an existing output directory:
- Content **outside** the markers (template-controlled) is regenerated fresh
- Content **inside** the markers (user-controlled) is preserved from the previous generation
- Multiple marker pairs per file are supported and matched positionally

**Use case:** Allow users to customize generated files (add dependencies, settings, code) without losing their changes when the template is updated.

**How it works:**
1. First generation: Template creates file with empty markers
2. User edits the file and adds content between markers
3. Re-run templator on same output directory
4. Template content outside markers is refreshed; user content inside markers survives

Example workflow:
```toml
# pyproject.toml (after first generation)
[project]
name = "my_app"

[dependencies]
# [preserve:begin]
# [preserve:end]
```

User edits to add dependencies:
```toml
[dependencies]
# [preserve:begin]
requests = ">=2.31"
pydantic = ">=2.0"
# [preserve:end]
```

On regeneration with updated template (e.g., new project name):
```toml
[project]
name = "my_updated_app"  ← refreshed from template

[dependencies]
# [preserve:begin]
requests = ">=2.31"      ← preserved from user edits
pydantic = ">=2.0"
# [preserve:end]
```

---

## Complete Feature Reference

### File Handling Rules

| Scope | Syntax | What Happens |
|---|---|---|
| **Directory names** | `__variable_name__` | Substituted with variable value; `.template` suffix stripped if present |
| **File names** | `__variable_name__` | Substituted with variable value; `.template` suffix stripped after rendering |
| **File content (.template)** | `{{ variable }}` | Rendered with Tera/Jinja2 syntax |
| **File content (non-.template)** | N/A | Copied byte-for-byte, never modified |

### Exclusion Patterns

Used in `pre_processing` steps with type `"exclude"`.

| Pattern | Matches | Examples |
|---|---|---|
| `*.ext` | Files with specific extension | `*.tmp`, `*.draft`, `*.bak` |
| `filename` | Exact filename (case-sensitive) | `.env.example`, `README.md` |
| `dir/` | Entire directory (trailing `/` required) | `build/`, `node_modules/`, `dist/` |
| `**/*.ext` | Files at any depth | `**/*.log`, `**/*.tmp` |
| `**/dirname/**` | Directory and all contents at any depth | `**/node_modules/**`, `**/.git/**` |

**Matching:**
- Paths normalized to forward slashes (cross-platform compatible)
- `**` matches zero or more path components
- `*` matches any characters within a single path component
- Multiple exclusion steps are additive (file excluded if it matches ANY pattern)

### Pre-Processing Features

Run before template generation. Organize and filter which files the engine sees.

| Feature | Scope | Syntax | Use Case |
|---|---|---|---|
| **exclude** | Step array | `{ "type": "exclude", "patterns": [...] }` | Glob patterns to skip during template generation |

**Constraints:**
- No way to un-exclude a file once excluded
- Exclusions happen before the engine walks the template

### Post-Processing Features

Run after template generation. Modify generated files, preserve user edits, or apply transformations.

#### Feature Comparison

| Feature | Input | Output | Preserves | Patterns | Multi-File |
|---|---|---|---|---|---|
| **replace** | Generated file content | Modified content | N/A (overwrites) | Literal strings (from/to) | Multiple steps, one file each |
| **preserve_sections** | Generated + pre-existing file | Merged content | User sections between markers | Marker-based | Multiple markers per file |

#### `replace` — When to Use

✅ Use for:
- Substituting author, version, or date placeholders
- Replacing build-system-specific values
- One-time string transformations

❌ Don't use for:
- Complex templating (use Tera in `.template` files instead)
- Conditional content (use Tera `{% if %}` blocks)
- Preserving user edits (use `preserve_sections` instead)

#### `preserve_sections` — When to Use

✅ Use for:
- Allowing users to add dependencies while template controls metadata
- Protecting custom configuration across regenerations
- Separating template-controlled and user-controlled sections
- Supporting multiple customization points in one file

❌ Don't use for:
- One-time substitutions (use `replace` instead)
- Complex transformations (pre-process with a script, then generate)
- Removing or filtering content (use `exclude` instead)

---

## Example template layout

```
examples/basic-template/
├── templator.json                  ← config file (auto-loaded)
└── template/
    ├── pyproject.toml.template     ← content rendered via Tera
    ├── .gitignore                  ← copied verbatim
    └── src/
        └── __project_name__/       ← directory renamed via __var__
            └── main.py.template    ← content rendered via Tera
```

With an answers file (answers.json):

```json
{
  "variables": {
    "project_name": "my_project",
    "python_version": "3.12",
    "author": "Alice"
  }
}
```

Running templator:

```bash
templator \
  --template examples/basic-template \
  --answers  answers.json \
  --output   /tmp/my-project
```

Produces:

```
/tmp/my-project/
├── pyproject.toml
├── .gitignore
└── src/
    └── my_project/
        └── main.py
```

## Security

- Variable values containing `..` or absolute paths are rejected before any file is written.
- `replace` post-processing paths are validated against path traversal and symlink escape via `canonicalize`.
- `.git` directories inside the template are silently skipped.
