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

#### `run_command` — run a shell command in the output directory

```json
{ "type": "run_command", "command": "chmod +x scripts/setup.sh" }
```

The command runs via `sh -c` with the output directory as the working directory.  
A non-zero exit code is a hard error.

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
