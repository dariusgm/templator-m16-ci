# Templator — Rust MVP Plan für Claude Code

**Tool:** Template-Engine für Projektgenerierung. Rust-Rewrite einer internen Python-Lösung.  
**Scope:** MVP am Wochenende (4–6h), Freitag–Samstag  
**Ziel:** Binary `templator`, <100ms generiert Projekt aus JSON + Template-Dir

---

## Quick Start für Claude Code

```bash
# Terminal in Claude Code
cargo new templator
cd templator
```

Dann: Schrittweise Dateien/Module wie in Plan 4 ausführen.

---

## 1. Cargo.toml Setup

```toml
[package]
name = "templator"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "templator"
path = "src/main.rs"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
tera = "1.20"
clap = { version = "4.5", features = ["derive"] }
walkdir = "2.4"
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"
env_logger = "0.11"

[dev-dependencies]
tempfile = "3.10"
```

---

## 2. Projektstruktur (erstelle diese Dateien)

```
src/
├── main.rs              # CLI entry + arg parsing
├── lib.rs               # Public API re-exports
├── config.rs            # Config structs + Serde
├── engine.rs            # Template rendering + file copy
├── post_process.rs      # Post-processing handlers
└── error.rs             # Custom error types (optional)

examples/
└── basic-template/
    ├── config.json
    └── template/
        ├── pyproject.toml
        ├── .gitignore
        └── src/
            └── __project_name__/
                └── main.py

tests/
└── integration_tests.rs
```

---

## 3. Datei-für-Datei Implementierung

### 3.1 `src/config.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub description: Option<String>,
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub post_processing: Vec<PostProcessStep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum PostProcessStep {
    #[serde(rename = "preserve_sections")]
    PreserveSections {
        file: String,
        marker: String,
    },
    #[serde(rename = "replace")]
    Replace {
        file: String,
        patterns: Vec<ReplacePattern>,
    },
    #[serde(rename = "run_command")]
    RunCommand {
        command: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplacePattern {
    pub from: String,
    pub to: String,
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
}
```

### 3.2 `src/engine.rs`

```rust
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tera::Tera;
use walkdir::WalkDir;
use crate::config::Config;

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new(template_dir: &Path) -> anyhow::Result<Self> {
        let glob_pattern = format!("{}/**/*", template_dir.display());
        let mut tera = Tera::new(&glob_pattern)?;
        Ok(Self { tera })
    }

    pub fn generate(
        &self,
        template_dir: &Path,
        output_dir: &Path,
        config: &Config,
    ) -> anyhow::Result<()> {
        // 1. Create output dir
        std::fs::create_dir_all(output_dir)?;

        // 2. Prepare Tera context
        let mut context = tera::Context::new();
        for (key, value) in &config.variables {
            context.insert(key, value);
        }

        // 3. Walk template dir and copy/render files
        for entry in WalkDir::new(template_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let rel_path = entry.path().strip_prefix(template_dir)?;
            
            // Skip .git and hidden dirs
            if rel_path.to_string_lossy().contains(".git") 
                || rel_path.to_string_lossy().starts_with(".") {
                continue;
            }

            let output_path = output_dir.join(rel_path);

            if entry.path().is_dir() {
                std::fs::create_dir_all(&output_path)?;
            } else {
                // Render filename (handles {{ variables }} in filenames)
                let rendered_filename = self.tera.render_str(
                    &rel_path.to_string_lossy(),
                    &context,
                )?;
                let final_output_path = output_dir.join(&rendered_filename);

                // Create parent dirs
                if let Some(parent) = final_output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Read, render, write
                let content = std::fs::read_to_string(entry.path())?;
                let rendered = self.tera.render_str(&content, &context)?;
                std::fs::write(&final_output_path, rendered)?;
                
                log::debug!("Rendered: {}", rendered_filename);
            }
        }

        Ok(())
    }
}
```

### 3.3 `src/post_process.rs`

```rust
use std::path::PathBuf;
use std::process::Command;
use crate::config::{PostProcessStep, ReplacePattern};

pub struct PostProcessor {
    working_dir: PathBuf,
}

impl PostProcessor {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    pub fn execute(&self, steps: &[PostProcessStep]) -> anyhow::Result<()> {
        for step in steps {
            match step {
                PostProcessStep::PreserveSections { file, marker } => {
                    self.preserve_sections(file, marker)?;
                }
                PostProcessStep::Replace { file, patterns } => {
                    self.replace_in_file(file, patterns)?;
                }
                PostProcessStep::RunCommand { command } => {
                    self.run_command(command)?;
                }
            }
        }
        Ok(())
    }

    fn preserve_sections(&self, file: &str, marker: &str) -> anyhow::Result<()> {
        // MVP: Placeholder
        // TODO: Merge Original-File Sections mit Template-File
        log::info!("Preserving sections in {} (marker: {})", file, marker);
        Ok(())
    }

    fn replace_in_file(&self, file: &str, patterns: &[ReplacePattern]) -> anyhow::Result<()> {
        let path = self.working_dir.join(file);
        let mut content = std::fs::read_to_string(&path)?;

        for pattern in patterns {
            content = content.replace(&pattern.from, &pattern.to);
            log::debug!("Replaced '{}' with '{}'", pattern.from, pattern.to);
        }

        std::fs::write(&path, content)?;
        log::info!("✓ Replaced patterns in {}", file);
        Ok(())
    }

    fn run_command(&self, command: &str) -> anyhow::Result<()> {
        log::info!("Running: {}", command);
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}\n{}", command, stderr);
        }

        log::info!("✓ Command completed");
        Ok(())
    }
}
```

### 3.4 `src/lib.rs`

```rust
pub mod config;
pub mod engine;
pub mod post_process;

pub use config::Config;
pub use engine::TemplateEngine;
pub use post_process::PostProcessor;
```

### 3.5 `src/main.rs`

```rust
use clap::Parser;
use std::path::PathBuf;
use templator::{Config, TemplateEngine, PostProcessor};

#[derive(Parser)]
#[command(name = "templator")]
#[command(about = "Generate projects from templates")]
#[command(version)]
struct Args {
    /// Template directory
    #[arg(short, long)]
    template: PathBuf,

    /// Config file (JSON)
    #[arg(short, long)]
    config: PathBuf,

    /// Output directory
    #[arg(short, long)]
    output: PathBuf,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    // Setup logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let args = Args::parse();

    // Validate inputs
    if !args.template.exists() {
        anyhow::bail!("Template directory not found: {}", args.template.display());
    }
    if !args.config.exists() {
        anyhow::bail!("Config file not found: {}", args.config.display());
    }

    // Load config
    let config = Config::from_file(&args.config)?;
    println!("Generating project: {}", config.name);

    // Generate from template
    let engine = TemplateEngine::new(&args.template)?;
    engine.generate(&args.template, &args.output, &config)?;
    println!("✓ Files generated");

    // Post-processing
    if !config.post_processing.is_empty() {
        let processor = PostProcessor::new(args.output.clone());
        processor.execute(&config.post_processing)?;
    }

    println!("✓ Project ready: {}", args.output.display());
    Ok(())
}
```

---

## 4. Example Template

### `examples/basic-template/config.json`

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

### `examples/basic-template/template/pyproject.toml`

```toml
[project]
name = "{{ project_name }}"
version = "0.1.0"
description = ""
authors = [{ name = "__AUTHOR__" }]

[tool.ruff]
target-version = "{{ python_version_short }}"

[build-system]
requires = ["setuptools"]
build-backend = "setuptools.build_meta"
```

### `examples/basic-template/template/.gitignore`

```
__pycache__/
*.pyc
.venv/
dist/
build/
*.egg-info/
.DS_Store
```

### `examples/basic-template/template/src/__project_name__/main.py`

```python
#!/usr/bin/env python3
"""
{{ project_name }} — {{ description or "Project" }}
Author: {{ author }}
"""

def main():
    print(f"Hello from {{ project_name }}!")

if __name__ == "__main__":
    main()
```

---

## 5. Integration Test

### `tests/integration_tests.rs`

```rust
use std::fs;
use std::path::PathBuf;
use templator::{Config, TemplateEngine, PostProcessor};

#[test]
fn test_basic_generation() -> anyhow::Result<()> {
    let template_dir = PathBuf::from("examples/basic-template/template");
    let config_file = PathBuf::from("examples/basic-template/config.json");
    
    let output_dir = tempfile::tempdir()?.path().to_path_buf();

    // Load and generate
    let config = Config::from_file(&config_file)?;
    let engine = TemplateEngine::new(&template_dir)?;
    engine.generate(&template_dir, &output_dir, &config)?;

    // Check outputs
    assert!(output_dir.join("pyproject.toml").exists());
    assert!(output_dir.join(".gitignore").exists());
    assert!(output_dir.join("src/my_project/main.py").exists());

    // Check content
    let pyproject = fs::read_to_string(output_dir.join("pyproject.toml"))?;
    assert!(pyproject.contains("name = \"my_project\""));
    assert!(pyproject.contains("py312"));

    Ok(())
}
```

---

## 6. Implementierungs-Schritte für Claude Code

### **Freitag Abend (1–2h)**

1. Terminal: `cargo new templator && cd templator`
2. Update `Cargo.toml` mit Dependencies
3. Erstelle `src/config.rs` (Serde structs)
4. Erstelle `src/lib.rs` (re-exports)
5. Erstelle `src/main.rs` (CLI skeleton)

→ Test: `cargo build` sollte kompilieren

### **Samstag Morgen (2–3h)**

6. Erstelle `src/engine.rs` (TemplateEngine)
7. Erstelle `src/post_process.rs` (PostProcessor)
8. Implementiere `main.rs` vollständig
9. Erstelle `examples/basic-template/` mit config.json + template files

→ Test:
```bash
cargo run -- \
  --template examples/basic-template/template \
  --config examples/basic-template/config.json \
  --output /tmp/test-output
```

### **Samstag Mittag (0.5–1h)**

10. Erstelle `tests/integration_tests.rs`
11. `cargo test` ausführen
12. Fehler beheben

### **Samstag Nachmittag (0.5h)**

13. `README.md` schreiben (Basic Usage + Example)
14. `cargo build --release`
15. Binary testen: `./target/release/templator --help`

---

## 7. Befehle zum Testen

```bash
# Build
cargo build --release

# Run with example
./target/release/templator \
  --template examples/basic-template/template \
  --config examples/basic-template/config.json \
  --output /tmp/test-output

# Check output
ls -la /tmp/test-output
cat /tmp/test-output/pyproject.toml

# Tests
cargo test

# Benchmark (einfach)
time ./target/release/templator --template ... --config ... --output ...
```

---

## 8. README.md Skeleton

```markdown
# Templator

Fast, simple template-based project generator. Written in Rust.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
templator --template /path/to/template \
          --config /path/to/config.json \
          --output /path/to/output
```

## Config Format

```json
{
  "name": "project-name",
  "variables": {
    "project_name": "my_project",
    "author": "You"
  },
  "post_processing": [
    { "type": "replace", "file": "pyproject.toml", "patterns": [...] },
    { "type": "run_command", "command": "./setup.sh" }
  ]
}
```

## Template Syntax

Variables in templates use Jinja2/Tera syntax:

```
Project: {{ project_name }}
Author: {{ author }}
```

Files and directories can also use variables:
- `src/__project_name__/` → `src/my_project/`

## Performance

- Typical project generation: <100ms
- Rust binary, zero runtime dependencies (on Linux/macOS)

## See Also

- Copier (Python, slower)
- Cookiecutter (Python, slower)
- Scaffold (Node.js, slower)
```

---

## 9. Success Checklist

- [ ] `cargo build --release` kompiliert fehlerlos
- [ ] `templator --help` zeigt Usage
- [ ] Example-Generation in <200ms
- [ ] Output hat richtige Variablen-Substitution
- [ ] post_processing (replace) funktioniert
- [ ] `cargo test` grün
- [ ] Binary läuft standalone (keine Dependencies)

---

## 10. Nach dem MVP

1. GitHub Repo + Release
2. Medium-Artikel: "How I Rewrote a Python Tool in Rust" (Apply darius-writing-style)
3. Später: preserve_sections implementieren
4. Später: Mehr Post-Processing-Typen wenn Bedarf da ist