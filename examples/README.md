# Templator Examples

## Complete Example

The `complete-example/` directory demonstrates **all** features of templator in a single coherent Python project template.

### Features Demonstrated

| Feature | Where it appears |
|---|---|
| **Variable substitution in filenames** | `src/__project_name__/` directory is renamed to `src/{{ project_name }}/` |
| **Tera template rendering** | `.template` files like `pyproject.toml.template`, `README.md.template`, and `src/__project_name__/main.py.template` |
| **Static file copying** | `.gitignore` is copied verbatim without processing |
| **File exclusion** | `*.draft`, `docs/private/`, `**/*.tmp`, and partial `.git/` directory exclusion |
| **Pre-processing exclusion** | `pre_processing` patterns exclude `*.log` and `temp/` |
| **`replace` post-processing** | `__COPYRIGHT_YEAR__` placeholder replaced with "2026" in `pyproject.toml` |
| **`preserve_sections` post-processing** | User-edited section in `[project.optional-dependencies]` survives regeneration |
| **Git hooks** | `.git/hooks/pre-commit` is included as an executable file |

### Running the Example

```bash
# Generate the project
cargo run -- \
  --template examples/complete-example/template \
  --config examples/complete-example-answers.json \
  --output /tmp/my-generated-project

# Examine the output
ls -la /tmp/my-generated-project
cat /tmp/my-generated-project/pyproject.toml
python /tmp/my-generated-project/src/my_project/main.py
```

### Files and Directory Layout

```
complete-example/
├── templator.json              # Config with all feature sections
├── template/
│   ├── src/
│   │   └── __project_name__/   # Renamed via variable substitution
│   │       └── main.py.template
│   ├── pyproject.toml.template # Tera + replace + preserve_sections
│   ├── README.md.template      # Tera rendering
│   ├── .gitignore              # Static file (no processing)
│   ├── docs/
│   │   ├── guide.md            # Included
│   │   └── private/
│   │       └── notes.md        # Excluded by "docs/private/"
│   ├── .git/
│   │   └── hooks/
│   │       └── pre-commit      # Executable; selective .git/ inclusion
│   ├── main.py.draft           # Excluded by "*.draft"
│   ├── temp/
│   │   └── scratch.txt         # Excluded by pre_processing "temp/"
│   └── debug.log               # Excluded by pre_processing "*.log"
└── ../complete-example-answers.json
```

---

## Reference: Post-Processing Types

### `replace` — Literal String Substitution

Applied **after** Tera rendering. Useful for content that cannot be rendered during template processing (e.g., copyright years, version numbers in multiple places).

```json
{
  "type": "replace",
  "file": "pyproject.toml",
  "patterns": [
    { "from": "__COPYRIGHT_YEAR__", "to": "2026" },
    { "from": "__VERSION__", "to": "0.1.0" }
  ]
}
```

### `preserve_sections` — Keep User Edits Across Regeneration

Marks a section of a file with begin/end markers. On regeneration, user-edited content between the markers is preserved when the template changes.

Usage:
1. First generation: template renders with empty markers.
2. User adds content between markers (e.g., dependencies in `[project.optional-dependencies]`).
3. Before regeneration, call `snapshot_preserved_sections()`.
4. After regeneration, call `execute()` to restore user content.

```json
{
  "type": "preserve_sections",
  "file": "pyproject.toml",
  "begin_marker": "# [preserve:begin]",
  "end_marker": "# [preserve:end]"
}
```

Markers in template:
```toml
[project.optional-dependencies]
# [preserve:begin]
# [preserve:end]
```

**Multiple sections per file:** A single file can have multiple begin/end marker pairs; each section is independently preserved.

---

## Reference: Exclusion Patterns

### Top-level `exclude`

```json
{
  "exclude": [
    "*.draft",           // All .draft files
    "docs/private/",     // Directory and all contents
    "build/",            // Another directory
    "**/*.tmp",          // All .tmp files at any depth
    ".git/config"        // Exact file match
  ]
}
```

Pattern syntax:
- `*.ext` — files ending with `.ext` in any directory
- `dir/` — entire directory and all its contents (trailing slash required)
- `**/*.ext` — files ending with `.ext` at any depth
- `exact/path/file` — exact path match
- `**/dir/**` — any depth to `dir/`, then any depth within

### Pre-processing `exclude`

In addition to top-level `exclude`, add exclude patterns via `pre_processing`:

```json
{
  "pre_processing": [
    {
      "type": "exclude",
      "patterns": ["*.log", "temp/", "docs/internal/"]
    }
  ]
}
```

These patterns are **merged** with top-level `exclude` — both apply.

---

## How Variables Work

### In Filenames (path substitution)

Any path component named `__varname__` is replaced with the variable value:

```
src/__project_name__/main.py  →  src/my_app/main.py
```

### In Template Files (Tera/Jinja2)

Only `.template` files are rendered. Use Jinja2/Tera syntax:

```jinja2
# This file is {{ name }}
Version: {{ version }}
Author: {{ author | default(value="Anonymous") }}
```

### In Non-Template Files

Files **without** `.template` suffix are copied verbatim. Tera syntax is **not** processed:

```
.gitignore (copied as-is)
```

---

## Security Considerations

- Variable values containing `..` or absolute paths are rejected.
- `replace` post-processing paths are validated against path traversal via `canonicalize`.
- `.git/` can be partially included by excluding specific internal paths (see complete-example).

For more details, see `CLAUDE.md` and source code comments in `src/post_process.rs`.
