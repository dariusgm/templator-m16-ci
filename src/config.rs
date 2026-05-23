use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub description: Option<String>,
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub pre_processing: Vec<PreProcessStep>,
    #[serde(default)]
    pub post_processing: Vec<PostProcessStep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum PostProcessStep {
    #[serde(rename = "preserve_sections")]
    PreserveSections { file: String, marker: String },
    #[serde(rename = "replace")]
    Replace {
        file: String,
        patterns: Vec<ReplacePattern>,
    },
    #[serde(rename = "run_command")]
    RunCommand { command: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplacePattern {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum PreProcessStep {
    #[serde(rename = "exclude")]
    Exclude { patterns: Vec<String> },
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read config {:?}: {}", path, e))?;
        serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid config JSON in {:?}: {}", path, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_minimal_config() {
        let json = r#"{"name": "test-project", "variables": {}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "test-project");
        assert!(config.description.is_none());
        assert!(config.post_processing.is_empty());
    }

    #[test]
    fn test_parse_full_config() {
        let json = r#"{
            "name": "my-project",
            "description": "A test project",
            "variables": {
                "project_name": "my_project",
                "author": "Alice"
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.description.as_deref(), Some("A test project"));
        assert_eq!(config.variables["author"], "Alice");
    }

    #[test]
    fn test_parse_replace_post_processing() {
        let json = r#"{
            "name": "test",
            "variables": {},
            "post_processing": [{
                "type": "replace",
                "file": "README.md",
                "patterns": [{"from": "__FOO__", "to": "bar"}]
            }]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        match &config.post_processing[0] {
            PostProcessStep::Replace { file, patterns } => {
                assert_eq!(file, "README.md");
                assert_eq!(patterns[0].from, "__FOO__");
                assert_eq!(patterns[0].to, "bar");
            }
            _ => panic!("Expected Replace step"),
        }
    }

    #[test]
    fn test_parse_run_command_post_processing() {
        let json = r#"{
            "name": "test",
            "variables": {},
            "post_processing": [{"type": "run_command", "command": "echo hello"}]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        match &config.post_processing[0] {
            PostProcessStep::RunCommand { command } => assert_eq!(command, "echo hello"),
            _ => panic!("Expected RunCommand step"),
        }
    }

    #[test]
    fn test_from_file() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, r#"{{"name": "x", "variables": {{}}}}"#).unwrap();
        let config = Config::from_file(f.path()).unwrap();
        assert_eq!(config.name, "x");
    }

    #[test]
    fn test_from_file_not_found() {
        let result = Config::from_file(Path::new("/nonexistent/templator.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_from_file_invalid_json_error_message() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{{ not valid json {{{{").unwrap();
        let err = Config::from_file(f.path()).unwrap_err();
        assert!(
            err.to_string().contains("Invalid config JSON"),
            "expected 'Invalid config JSON' in error, got: {err}"
        );
    }

    #[test]
    fn test_parse_null_name_errors() {
        let json = r#"{"name": null, "variables": {}}"#;
        assert!(serde_json::from_str::<Config>(json).is_err());
    }

    #[test]
    fn test_parse_unknown_post_process_type_errors() {
        let json = r#"{"name":"t","variables":{},"post_processing":[{"type":"unknown_step"}]}"#;
        assert!(serde_json::from_str::<Config>(json).is_err());
    }

    #[test]
    fn test_parse_exclude_patterns() {
        let json = r#"{
            "name": "test",
            "variables": {},
            "exclude": ["*.tmp", "build/", "node_modules/**"]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.exclude.len(), 3);
        assert_eq!(config.exclude[0], "*.tmp");
        assert_eq!(config.exclude[1], "build/");
        assert_eq!(config.exclude[2], "node_modules/**");
    }

    #[test]
    fn test_parse_exclude_defaults_to_empty() {
        let json = r#"{"name": "test", "variables": {}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_parse_pre_processing_defaults_to_empty() {
        let json = r#"{"name": "test", "variables": {}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.pre_processing.is_empty());
    }

    #[test]
    fn test_parse_pre_processing_exclude_step() {
        let json = r#"{
            "name": "test",
            "variables": {},
            "pre_processing": [{
                "type": "exclude",
                "patterns": ["*.log", "temp/"]
            }]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.pre_processing.len(), 1);
        match &config.pre_processing[0] {
            PreProcessStep::Exclude { patterns } => {
                assert_eq!(patterns.len(), 2);
                assert_eq!(patterns[0], "*.log");
                assert_eq!(patterns[1], "temp/");
            }
        }
    }

    #[test]
    fn test_parse_unknown_pre_process_type_errors() {
        let json = r#"{"name":"t","variables":{},"pre_processing":[{"type":"unknown_step"}]}"#;
        assert!(serde_json::from_str::<Config>(json).is_err());
    }
}
