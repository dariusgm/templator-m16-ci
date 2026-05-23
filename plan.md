# Templator — Full Breaking Change Implementation Plan

## The Model (Copier-Aligned)

**One way, no options:**

```bash
templator --template ./template \
          --answers ./my-answers.json \
          --output ./my-project
```

- `./template/templator.json` **must exist** (auto-discovered)
- `./my-answers.json` **must be provided** (no prompts, no defaults)
- `./my-project/.templator-answers.json` is **always written**

No ambiguity. No legacy paths. Just this.

---

## Implementation Checklist

### 1. Rewrite `src/main.rs` (CLI Layer)

Replace the entire CLI argument parsing:

```rust
use clap::Parser;

#[derive(Parser)]
#[command(about = "Template engine with variable substitution")]
struct Args {
    /// Template directory (must contain templator.json)
    #[arg(long)]
    template: PathBuf,

    /// Project-specific answers (JSON file)
    #[arg(long)]
    answers: PathBuf,

    /// Output directory
    #[arg(long)]
    output: PathBuf,
}
```

**No `--config` flag exists anymore.**

**Validation on startup:**
- `template/templator.json` exists → if not, error and exit
- `answers` is valid JSON → if not, error and exit
- All required variables from answers exist → if not, error and exit

**Then orchestrate:**
1. Load `template/templator.json`
2. Load answers JSON
3. Run `TemplateEngine`
4. Run post-processing
5. Write `.templator-answers.json` to output

---

### 2. Rewrite `src/config.rs` (Data Models)

**Split into two clear structs:**

```rust
/// Template definition (from templator.json in template directory)
#[derive(Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub variables: Map<String, VariableDefinition>,
    pub post_processing: Option<Vec<PostProcessStep>>,
}

/// Variable structure
#[derive(Deserialize)]
pub struct VariableDefinition {
    pub r#type: String,  // "string", "integer", etc.
    pub default: Option<String>,
}

/// Project answers (from --answers file)
#[derive(Deserialize, Serialize)]
pub struct ProjectAnswers {
    pub variables: Map<String, serde_json::Value>,
}

/// For writing to output
#[derive(Serialize)]
pub struct SavedAnswers {
    pub _template: String,  // Reference to template path
    pub _generated_at: String,  // ISO timestamp
    #[serde(flatten)]
    pub answers: ProjectAnswers,
}
```

**Load functions:**
```rust
impl TemplateConfig {
    /// Load from template_dir/templator.json
    /// Fails if not found
    pub fn load_from_template_dir(template_dir: &Path) -> Result<Self> {
        let path = template_dir.join("templator.json");
        if !path.exists() {
            return Err(format!(
                "Template config not found: {}",
                path.display()
            ));
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

impl ProjectAnswers {
    /// Load from answers JSON file
    pub fn load_from_file(answers_file: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(answers_file)?;
        Ok(serde_json::from_str(&content)?)
    }
    
    /// Validate that all required variables are present
    pub fn validate_against(&self, config: &TemplateConfig) -> Result<()> {
        for (var_name, _var_def) in &config.variables {
            if !self.variables.contains_key(var_name) {
                return Err(format!("Required variable missing: {}", var_name));
            }
        }
        Ok(())
    }
}

impl SavedAnswers {
    /// Save to output_dir/.templator-answers.json
    pub fn save_to_output(&self, output_dir: &Path) -> Result<()> {
        let path = output_dir.join(".templator-answers.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

---

### 3. Update `src/engine.rs` (Template Processing)

**New signature:**

```rust
pub struct TemplateEngine {
    template_dir: PathBuf,
    config: TemplateConfig,
    answers: ProjectAnswers,
}

impl TemplateEngine {
    pub fn new(
        template_dir: PathBuf,
        config: TemplateConfig,
        answers: ProjectAnswers,
    ) -> Self {
        Self {
            template_dir,
            config,
            answers,
        }
    }

    /// Process template directory and write to output
    pub fn render(&self, output_dir: &Path) -> Result<()> {
        // 1. Walk template_dir/template/ (or just template_dir if no subdir)
        // 2. For each file:
        //    - Replace __var__ in filename
        //    - If .template suffix: render with Jinja2
        //    - Else: copy as-is
        // 3. Post-process if configured
        // 4. Write .templator-answers.json
        todo!()
    }
}
```

**No breaking changes to internal logic** — just cleaner argument passing.

---

### 4. Post-Processing Unchanged

`src/post_process.rs` stays the same. It just receives the merged config.

---

### 5. Update Tests

**File: `tests/integration_tests.rs`**

Change fixture:
```
examples/basic-template/
├── templator.json          ← (moved here, no longer external)
├── template/
│   ├── pyproject.toml.template
│   ├── src/__project_name__/main.py.template
│   └── .gitignore

examples/basic-answers.json ← (separate file)
{
  "variables": {
    "project_name": "my_cli",
    "python_version": "3.11"
  }
}
```

Test cases:
```rust
#[test]
fn test_basic_generation() {
    let result = templator(
        "--template=examples/basic-template",
        "--answers=examples/basic-answers.json",
        "--output=target/test-output/basic",
    );
    assert!(result.is_ok());
    assert!(Path::new("target/test-output/basic/.templator-answers.json").exists());
}

#[test]
fn test_missing_templator_json() {
    let result = templator(
        "--template=target/nonexistent",
        "--answers=examples/basic-answers.json",
        "--output=target/test-output",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("templator.json"));
}

#[test]
fn test_missing_required_variable() {
    // answers.json is missing a required variable
    let result = templator(
        "--template=examples/basic-template",
        "--answers=examples/incomplete-answers.json",
        "--output=target/test-output",
    );
    assert!(result.is_err());
}
```

---

### 6. Update Documentation

**README.md:**
- Remove any mention of `--config`
- Show only the new `--template --answers --output` pattern
- Explain file layout clearly

**CLAUDE.md:**
- No changes

**PROJECT.md:**
- Update descriptions of `src/config.rs` and `src/main.rs`

---

## File-by-File Summary

| File | Change |
|---|---|
| `src/main.rs` | Complete rewrite: new CLI, new orchestration |
| `src/config.rs` | New structs: `TemplateConfig`, `ProjectAnswers`, `SavedAnswers` |
| `src/engine.rs` | Minimal changes: accept new config structs |
| `src/post_process.rs` | No changes |
| `src/lib.rs` | Update exports if needed |
| `tests/integration_tests.rs` | Update fixtures and test cases |
| `examples/basic-template/templator.json` | Move from outside to inside template dir |
| `examples/basic-answers.json` | New file: project-specific answers |
| `README.md` | Update examples and workflow |
| `Cargo.toml` | No changes |

---

## Execution Order

1. **Start with `src/main.rs`** — Define new CLI and error handling
2. **Then `src/config.rs`** — Build the data models
3. **Then `src/engine.rs`** — Wire them together
4. **Then `tests/integration_tests.rs`** — Fix fixtures and tests
5. **Then `README.md`** — Document the new workflow

Once everything compiles:
```bash
cargo fmt
cargo clippy
cargo test
```

---

## Success Criteria

- [ ] `templator --template ./t --answers ./a.json --output ./o` works
- [ ] `./t/templator.json` is auto-discovered
- [ ] `.templator-answers.json` is written to output with correct format
- [ ] All tests pass
- [ ] `cargo clippy` is clean
- [ ] README shows new workflow only
- [ ] No legacy code paths remain****