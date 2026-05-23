use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

use crate::config::TemplateConfig;
use crate::pre_process::PreProcessor;

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        template_dir: &Path,
        output_dir: &Path,
        config: &TemplateConfig,
        variables: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        std::fs::create_dir_all(output_dir)?;

        let mut context = tera::Context::new();
        // Inject top-level config fields so templates can reference {{ name }}, {{ description }}
        context.insert("name", &config.name);
        if let Some(ref desc) = config.description {
            context.insert("description", desc);
        }
        // Variables override config fields if names collide
        for (key, value) in variables {
            context.insert(key, value);
        }

        // Compute the effective exclude list by merging config.exclude with pre_processing patterns
        let effective_exclude =
            PreProcessor::new().effective_exclude(&config.exclude, &config.pre_processing);

        for entry in WalkDir::new(template_dir).into_iter() {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("Skipping unreadable entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();
            let rel_path = match path.strip_prefix(template_dir) {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Skip the template root itself
            if rel_path == Path::new("") {
                continue;
            }

            // Skip excluded paths
            if is_excluded(rel_path, &effective_exclude) {
                log::debug!("Excluded: {}", rel_path.display());
                continue;
            }

            // Apply __var__ substitution to path components, then strip .template suffix
            let substituted = substitute_path(rel_path, variables);

            // Reject variable values that would escape the output directory
            if substituted.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            }) {
                anyhow::bail!(
                    "Unsafe path after variable substitution: '{}' — variable values must not contain '..' or absolute paths",
                    substituted.display()
                );
            }

            let output_rel = strip_template_suffix(&substituted);
            let output_path = output_dir.join(&output_rel);

            if path.is_dir() {
                std::fs::create_dir_all(&output_path)?;
            } else {
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                if has_template_suffix(path) {
                    // Render content with Tera — only for .template files
                    let content = std::fs::read_to_string(path)
                        .map_err(|e| anyhow::anyhow!("Cannot read {:?}: {}", path, e))?;
                    let rendered = tera::Tera::one_off(&content, &context, false)
                        .map_err(|e| anyhow::anyhow!("Render error in {:?}: {}", path, e))?;
                    std::fs::write(&output_path, rendered)?;
                    log::debug!("Rendered: {}", output_rel.display());
                } else {
                    // Copy verbatim — no content substitution
                    std::fs::copy(path, &output_path)?;
                    log::debug!("Copied:   {}", output_rel.display());
                }
            }
        }

        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns true if the path ends with the `.template` extension.
pub fn has_template_suffix(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "template")
}

/// Strips `.template` from every component of a path.
/// `foo.py.template` → `foo.py`, `assets.template/logo.png` → `assets/logo.png`.
pub fn strip_template_suffix(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        let s = component.as_os_str();
        let p = Path::new(s);
        if has_template_suffix(p) {
            result.push(p.with_extension(""));
        } else {
            result.push(s);
        }
    }
    result
}

/// Replaces `__key__` placeholders in every path component with the matching variable value.
pub fn substitute_path(path: &Path, vars: &HashMap<String, String>) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        let s = component.as_os_str().to_string_lossy();
        let mut part = s.to_string();
        for (key, value) in vars {
            part = part.replace(&format!("__{key}__"), value);
        }
        result.push(part);
    }
    result
}

/// Check if a path matches any exclude pattern.
/// Supports simple glob patterns: `*` matches any characters in a single component,
/// `**` matches any number of path components.
/// Paths are normalized to use forward slashes for cross-platform compatibility.
pub fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    let path_str = normalize_path(&path.to_string_lossy());
    for pattern in patterns {
        if matches_pattern(&path_str, pattern) {
            return true;
        }
    }
    false
}

/// Normalize a path to use forward slashes regardless of OS.
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Match a path against a simple glob pattern.
fn matches_pattern(path: &str, pattern: &str) -> bool {
    // Exact match
    if path == pattern {
        return true;
    }

    // Directory exclusion (e.g., "build/" or "node_modules/")
    if let Some(dir_pattern) = pattern.strip_suffix('/') {
        return path == dir_pattern || path.starts_with(&format!("{}/", dir_pattern));
    }

    // Recursive wildcard: "**" matches everything
    if pattern == "**" {
        return true;
    }

    // Pattern with **/ prefix: "**/*.tmp" matches .tmp files at any depth
    if let Some(suffix) = pattern.strip_prefix("**/") {
        if let Some(extension) = suffix.strip_prefix('*') {
            // Handle patterns like "**/*.tmp"
            return path.ends_with(extension);
        } else {
            // Handle patterns like "**/node_modules/**"
            if let Some(dir) = suffix.strip_suffix("/**") {
                return path == dir
                    || path.starts_with(&format!("{}/", dir))
                    || path.contains(&format!("/{}/", dir));
            } else {
                // Handle patterns like "**/specific/path"
                return path.ends_with(suffix) || path.contains(&format!("/{}", suffix));
            }
        }
    }

    // Wildcard in filename: e.g., "*.tmp" or "*.bak"
    if let Some(suffix) = pattern.strip_prefix('*') {
        if !pattern.contains('/') {
            return path.ends_with(suffix);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_config() -> TemplateConfig {
        TemplateConfig {
            name: "test".into(),
            description: None,
            variables: HashMap::new(),
            exclude: vec![],
            pre_processing: vec![],
            post_processing: vec![],
        }
    }

    fn make_variables(vars: &[(&str, &str)]) -> HashMap<String, String> {
        vars.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // --- has_template_suffix ---

    #[test]
    fn test_template_suffix_detected() {
        assert!(has_template_suffix(Path::new("main.py.template")));
        assert!(has_template_suffix(Path::new("pyproject.toml.template")));
        assert!(has_template_suffix(Path::new("foo.template")));
    }

    #[test]
    fn test_no_template_suffix_not_detected() {
        assert!(!has_template_suffix(Path::new(".gitignore")));
        assert!(!has_template_suffix(Path::new("main.py")));
        assert!(!has_template_suffix(Path::new("templator.json")));
        assert!(!has_template_suffix(Path::new("Makefile")));
    }

    // --- strip_template_suffix ---

    #[test]
    fn test_strip_double_extension() {
        assert_eq!(
            strip_template_suffix(Path::new("main.py.template")),
            PathBuf::from("main.py")
        );
        assert_eq!(
            strip_template_suffix(Path::new("pyproject.toml.template")),
            PathBuf::from("pyproject.toml")
        );
    }

    #[test]
    fn test_strip_single_extension() {
        assert_eq!(
            strip_template_suffix(Path::new("foo.template")),
            PathBuf::from("foo")
        );
    }

    #[test]
    fn test_strip_noop_for_non_template() {
        assert_eq!(
            strip_template_suffix(Path::new(".gitignore")),
            PathBuf::from(".gitignore")
        );
        assert_eq!(
            strip_template_suffix(Path::new("main.py")),
            PathBuf::from("main.py")
        );
    }

    #[test]
    fn test_strip_intermediate_directory_component() {
        // The .template suffix is stripped from every component, not just the last filename.
        assert_eq!(
            strip_template_suffix(Path::new("assets.template/logo.png")),
            PathBuf::from("assets/logo.png")
        );
        assert_eq!(
            strip_template_suffix(Path::new("src/views.template/index.html.template")),
            PathBuf::from("src/views/index.html")
        );
    }

    // --- substitute_path ---

    #[test]
    fn test_substitute_path_dir_component() {
        let vars = HashMap::from([("project_name".to_string(), "my_app".to_string())]);
        assert_eq!(
            substitute_path(Path::new("src/__project_name__"), &vars),
            PathBuf::from("src/my_app")
        );
    }

    #[test]
    fn test_substitute_path_filename() {
        let vars = HashMap::from([("module".to_string(), "utils".to_string())]);
        assert_eq!(
            substitute_path(Path::new("__module__.py.template"), &vars),
            PathBuf::from("utils.py.template")
        );
    }

    #[test]
    fn test_substitute_path_no_match() {
        let vars = HashMap::from([("foo".to_string(), "bar".to_string())]);
        assert_eq!(
            substitute_path(Path::new("README.md"), &vars),
            PathBuf::from("README.md")
        );
    }

    #[test]
    fn test_substitute_path_empty_vars() {
        let vars = HashMap::new();
        assert_eq!(
            substitute_path(Path::new("src/__project_name__/main.py"), &vars),
            PathBuf::from("src/__project_name__/main.py")
        );
    }

    // --- generate: .template files are rendered ---

    #[test]
    fn test_generate_renders_template_file() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::write(
            template_dir.path().join("hello.txt.template"),
            "Hello {{ name }}!",
        )
        .unwrap();

        let config = make_config();
        let vars = make_variables(&[("name", "World")]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert_eq!(
            fs::read_to_string(output_dir.path().join("hello.txt")).unwrap(),
            "Hello World!"
        );
        assert!(!output_dir.path().join("hello.txt.template").exists());
    }

    // --- generate: non-.template files are copied verbatim ---

    #[test]
    fn test_generate_copies_non_template_verbatim() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let raw = "{{ not_rendered }}\n__neither__\n";
        fs::write(template_dir.path().join("verbatim.txt"), raw).unwrap();

        let config = make_config();
        let vars = make_variables(&[]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        let out = fs::read_to_string(output_dir.path().join("verbatim.txt")).unwrap();
        assert_eq!(out, raw);
    }

    // --- generate: __var__ substitution in path ---

    #[test]
    fn test_generate_path_substitution() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let src = template_dir.path().join("src").join("__project_name__");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.py.template"), "# {{ project_name }}").unwrap();

        let config = make_config();
        let vars = make_variables(&[("project_name", "my_app")]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        let expected = output_dir.path().join("src").join("my_app").join("main.py");
        assert!(expected.exists(), "Path substitution failed");
        assert_eq!(fs::read_to_string(expected).unwrap(), "# my_app");
    }

    // --- generate: description injected into context ---

    #[test]
    fn test_generate_description_in_context() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::write(
            template_dir.path().join("README.md.template"),
            "# {{ name }}: {{ description }}",
        )
        .unwrap();

        let mut config = make_config();
        let vars = make_variables(&[]);
        config.description = Some("A great tool".into());

        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        let out = fs::read_to_string(output_dir.path().join("README.md")).unwrap();
        assert_eq!(out, "# test: A great tool");
    }

    // --- strip_template_suffix: double .template suffix ---

    #[test]
    fn test_strip_double_template_suffix_gives_template_output() {
        // foo.template.template → content rendered, output named foo.template
        assert_eq!(
            strip_template_suffix(Path::new("foo.template.template")),
            PathBuf::from("foo.template")
        );
    }

    #[test]
    fn test_generate_double_template_suffix_outputs_template_file() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::write(
            template_dir.path().join("out.txt.template.template"),
            "rendered: {{ val }}",
        )
        .unwrap();

        let config = make_config();
        let vars = make_variables(&[("val", "ok")]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        let output_file = output_dir.path().join("out.txt.template");
        assert!(output_file.exists(), ".template output file not found");
        assert_eq!(fs::read_to_string(output_file).unwrap(), "rendered: ok");
        assert!(!output_dir.path().join("out.txt.template.template").exists());
    }

    // --- generate: path traversal via variable values is rejected ---

    #[test]
    fn test_generate_rejects_dotdot_in_variable_value() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let src = template_dir.path().join("__unsafe__");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("file.txt"), "content").unwrap();

        let config = make_config();
        let vars = make_variables(&[("unsafe", "../../escaped")]);
        let result =
            TemplateEngine::new().generate(template_dir.path(), output_dir.path(), &config, &vars);
        assert!(
            result.is_err(),
            "Expected error for path traversal via variable"
        );
    }

    #[test]
    fn test_generate_allows_subdir_via_variable() {
        // Variable values with '/' that create subdirectories are fine (e.g. Java packages)
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let src = template_dir.path().join("src").join("__pkg__");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.py.template"), "# {{ pkg }}").unwrap();

        let config = make_config();
        let vars = make_variables(&[("pkg", "myorg/myapp")]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        let expected = output_dir.path().join("src/myorg/myapp/main.py");
        assert!(expected.exists(), "Subdirectory via variable failed");
    }

    // --- substitute_path: multiple vars in one component ---

    #[test]
    fn test_substitute_path_multiple_vars_in_one_component() {
        let vars = HashMap::from([
            ("org".to_string(), "acme".to_string()),
            ("project".to_string(), "widget".to_string()),
        ]);
        assert_eq!(
            substitute_path(Path::new("__org__-__project__"), &vars),
            PathBuf::from("acme-widget")
        );
    }

    #[test]
    fn test_substitute_path_empty_string_value() {
        let vars = HashMap::from([("prefix".to_string(), "".to_string())]);
        assert_eq!(
            substitute_path(Path::new("__prefix__main.py"), &vars),
            PathBuf::from("main.py")
        );
    }

    // --- generate: Tera syntax error returns error ---

    #[test]
    fn test_generate_tera_syntax_error_returns_error() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::write(
            template_dir.path().join("broken.txt.template"),
            "{{ unclosed",
        )
        .unwrap();

        let config = make_config();
        let vars = make_variables(&[]);
        let result =
            TemplateEngine::new().generate(template_dir.path(), output_dir.path(), &config, &vars);
        assert!(result.is_err(), "Invalid Tera syntax should return error");
    }

    // --- generate: empty template directory ---

    #[test]
    fn test_generate_empty_template_dir_succeeds() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let config = make_config();
        let vars = make_variables(&[]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        let entries: Vec<_> = fs::read_dir(output_dir.path()).unwrap().collect();
        assert!(
            entries.is_empty(),
            "Output should be empty for empty template"
        );
    }

    // --- generate: .git directories are skipped ---

    #[test]
    fn test_generate_includes_git_hooks_when_not_excluded() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let git_hooks_dir = template_dir.path().join(".git/hooks");
        fs::create_dir_all(&git_hooks_dir).unwrap();
        fs::write(git_hooks_dir.join("pre-commit"), "#!/bin/sh\necho ok").unwrap();
        fs::write(template_dir.path().join("README.md"), "hello").unwrap();

        let config = make_config();
        let vars = make_variables(&[]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert!(output_dir.path().join("README.md").exists());
        assert!(
            output_dir.path().join(".git/hooks/pre-commit").exists(),
            ".git/hooks should be copied when not excluded"
        );
    }

    #[test]
    fn test_generate_excludes_dot_git_when_excluded() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let git_dir = template_dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "[core]\nbare = false").unwrap();
        fs::write(template_dir.path().join("README.md"), "hello").unwrap();

        let mut config = make_config();
        config.exclude = vec![".git/".to_string()];
        let vars = make_variables(&[]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert!(output_dir.path().join("README.md").exists());
        assert!(
            !output_dir.path().join(".git").exists(),
            ".git dir should be excluded when matched"
        );
    }

    // --- generate: variable overrides config.name in Tera context ---

    #[test]
    fn test_generate_variable_overrides_config_name() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::write(template_dir.path().join("out.txt.template"), "{{ name }}").unwrap();

        // config.name = "test" (from make_config), variables["name"] = "overridden"
        let config = make_config();
        let vars = make_variables(&[("name", "overridden")]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert_eq!(
            fs::read_to_string(output_dir.path().join("out.txt")).unwrap(),
            "overridden"
        );
    }

    // --- generate: directory with .template suffix is renamed ---

    #[test]
    fn test_generate_directory_with_template_suffix_is_renamed() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let subdir = template_dir.path().join("assets.template");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("logo.png"), "binary").unwrap();

        let config = make_config();
        let vars = make_variables(&[]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert!(
            output_dir.path().join("assets").is_dir(),
            "assets.template dir should become assets/"
        );
        assert!(output_dir.path().join("assets/logo.png").exists());
        assert!(!output_dir.path().join("assets.template").exists());
    }

    // --- generate: tera default filter works ---

    #[test]
    fn test_generate_tera_default_filter() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::write(
            template_dir.path().join("out.txt.template"),
            "{{ missing_var | default(value=\"fallback\") }}",
        )
        .unwrap();

        let config = make_config();
        let vars = make_variables(&[]);
        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert_eq!(
            fs::read_to_string(output_dir.path().join("out.txt")).unwrap(),
            "fallback"
        );
    }

    // --- matches_pattern: glob pattern matching ---

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("file.txt", "file.txt"));
        assert!(!matches_pattern("file.txt", "other.txt"));
    }

    #[test]
    fn test_matches_pattern_extension() {
        assert!(matches_pattern("file.tmp", "*.tmp"));
        assert!(matches_pattern("path/to/file.bak", "*.bak"));
        assert!(!matches_pattern("file.txt", "*.tmp"));
    }

    #[test]
    fn test_matches_pattern_directory() {
        assert!(matches_pattern("build/file.o", "build/"));
        assert!(matches_pattern("build/nested/file.o", "build/"));
        assert!(!matches_pattern("src/file.rs", "build/"));
    }

    #[test]
    fn test_matches_pattern_recursive_wildcard() {
        assert!(matches_pattern("anything", "**"));
        assert!(matches_pattern("deeply/nested/path", "**"));
    }

    #[test]
    fn test_matches_pattern_recursive_with_suffix() {
        assert!(matches_pattern("node_modules/pkg/index.js", "**/*.js"));
        assert!(matches_pattern("deep/nested/file.tmp", "**/*.tmp"));
        assert!(!matches_pattern("file.rs", "**/*.js"));
    }

    // --- is_excluded: path exclusion ---

    #[test]
    fn test_is_excluded_empty_patterns() {
        assert!(!is_excluded(Path::new("file.txt"), &[]));
        assert!(!is_excluded(Path::new("build/artifact"), &[]));
    }

    #[test]
    fn test_is_excluded_single_pattern() {
        let patterns = vec!["*.tmp".to_string()];
        assert!(is_excluded(Path::new("file.tmp"), &patterns));
        assert!(!is_excluded(Path::new("file.txt"), &patterns));
    }

    #[test]
    fn test_is_excluded_multiple_patterns() {
        let patterns = vec![
            "*.tmp".to_string(),
            "build/".to_string(),
            "**/*.bak".to_string(),
        ];
        assert!(is_excluded(Path::new("file.tmp"), &patterns));
        assert!(is_excluded(Path::new("build/artifact"), &patterns));
        assert!(is_excluded(Path::new("src/old.bak"), &patterns));
        assert!(!is_excluded(Path::new("src/main.rs"), &patterns));
    }

    // --- generate: exclude patterns prevent file copying ---

    #[test]
    fn test_generate_excludes_files_by_extension() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::write(template_dir.path().join("keep.txt"), "keep").unwrap();
        fs::write(template_dir.path().join("skip.tmp"), "skip").unwrap();

        let mut config = make_config();
        let vars = make_variables(&[]);
        config.exclude = vec!["*.tmp".to_string()];

        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert!(
            output_dir.path().join("keep.txt").exists(),
            "keep.txt should exist"
        );
        assert!(
            !output_dir.path().join("skip.tmp").exists(),
            "skip.tmp should be excluded"
        );
    }

    #[test]
    fn test_generate_excludes_entire_directories() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::create_dir(template_dir.path().join("src")).unwrap();
        fs::create_dir(template_dir.path().join("build")).unwrap();

        fs::write(template_dir.path().join("src/main.rs"), "code").unwrap();
        fs::write(template_dir.path().join("build/artifact"), "binary").unwrap();

        let mut config = make_config();
        let vars = make_variables(&[]);
        config.exclude = vec!["build/".to_string()];

        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert!(output_dir.path().join("src").exists(), "src/ should exist");
        assert!(
            output_dir.path().join("src/main.rs").exists(),
            "src/main.rs should exist"
        );
        assert!(
            !output_dir.path().join("build").exists(),
            "build/ should be excluded"
        );
    }

    #[test]
    fn test_generate_excludes_recursive_patterns() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::create_dir_all(template_dir.path().join("src")).unwrap();
        fs::create_dir_all(template_dir.path().join("node_modules/pkg")).unwrap();

        fs::write(
            template_dir.path().join("node_modules/pkg/index.js"),
            "js code",
        )
        .unwrap();
        fs::write(template_dir.path().join("src/main.js"), "code").unwrap();

        let mut config = make_config();
        let vars = make_variables(&[]);
        config.exclude = vec!["node_modules/".to_string()];

        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert!(
            output_dir.path().join("src/main.js").exists(),
            "src/main.js should exist"
        );
        assert!(
            !output_dir.path().join("node_modules").exists(),
            "node_modules should be excluded"
        );
    }

    #[test]
    fn test_generate_excludes_template_files() {
        let template_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        fs::write(
            template_dir.path().join("config.template"),
            "excluded: {{ value }}",
        )
        .unwrap();
        fs::write(
            template_dir.path().join("README.template"),
            "keep: {{ value }}",
        )
        .unwrap();

        let mut config = make_config();
        let vars = make_variables(&[("value", "test")]);
        config.exclude = vec!["config.template".to_string()];

        TemplateEngine::new()
            .generate(template_dir.path(), output_dir.path(), &config, &vars)
            .unwrap();

        assert!(
            !output_dir.path().join("config").exists(),
            "config (from config.template) should be excluded"
        );
        assert_eq!(
            fs::read_to_string(output_dir.path().join("README")).unwrap(),
            "keep: test"
        );
    }
}
