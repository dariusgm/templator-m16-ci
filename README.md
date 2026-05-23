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
  --config   path/to/templator.json \
  --output   path/to/output

# Flags
#   -t, --template   Template directory
#   -c, --config     JSON config file (templator.json)
#   -o, --output     Output directory (created if it does not exist)
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

### File exclusion

Files and directories can be excluded from the output using glob patterns in the `exclude` array:

| Pattern | Matches | Example |
|---|---|---|
| `*.ext` | Files with specific extension | `*.tmp`, `*.draft` |
| `filename` | Exact filename | `.env.example` |
| `dir/` | Entire directory (requires trailing `/`) | `build/`, `node_modules/` |
| `**/*.ext` | Files at any depth | `**/*.log` matches `src/logs/debug.log` |

Excluded files are not copied to the output directory, allowing you to keep sensitive files, build artifacts, and temporary files in the template while generating clean projects.

### Pre-processing

`pre_processing` steps run before the engine walks the template directory. Currently, only the `exclude` step type is supported.

#### `exclude` — additional glob patterns

```json
"pre_processing": [
  {
    "type": "exclude",
    "patterns": ["*.log", "temp/", "docs/internal/"]
  }
]
```

Patterns are merged with any patterns already listed in the top-level `exclude` array.
Both are additive: a file is excluded if it matches any pattern from either source.

| Field | Type | Required | Description |
|---|---|---|---|
| `type` | string | yes | Must be `"exclude"` |
| `patterns` | array of strings | yes | Glob patterns using the same syntax as `exclude` |

**Why use `pre_processing.exclude` instead of `exclude`?**
- Organisational: group exclusions that belong together (e.g., all CI artifacts in one step, all draft files in another).
- Future-proofing: additional `pre_processing` step types will run in the same phase, letting you express more complex setup logic.

Both mechanisms co-exist. You can use one, the other, or both simultaneously.

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
  "exclude": [
    "*.draft",
    ".env.example"
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
| `exclude` | array | no | Glob patterns for files/directories to exclude from output |
| `pre_processing` | array | no | Steps to run before file generation |
| `post_processing` | array | no | Steps to run after file generation |

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

## Example template layout

```
examples/basic-template/
├── templator.json
└── template/
    ├── pyproject.toml.template     ← content rendered via Tera
    ├── .gitignore                  ← copied verbatim
    └── src/
        └── __project_name__/       ← directory renamed via __var__
            └── main.py.template    ← content rendered via Tera
```

Running against this example:

```bash
templator \
  --template examples/basic-template/template \
  --config   examples/basic-template/templator.json \
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
