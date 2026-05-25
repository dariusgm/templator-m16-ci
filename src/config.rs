use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct VariableDefinition {
    pub default: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub description: Option<String>,
    pub variables: HashMap<String, VariableDefinition>,
    #[serde(default)]
    pub pre_processing: Vec<PreProcessStep>,
    #[serde(default)]
    pub post_processing: Vec<PostProcessStep>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectAnswers {
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SavedAnswers {
    pub _template: String,
    pub _generated_at: String,
    #[serde(flatten)]
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum PostProcessStep {
    #[serde(rename = "preserve_sections")]
    PreserveSections {
        file: String,
        begin_marker: String,
        end_marker: String,
    },
    #[serde(rename = "replace")]
    Replace {
        file: String,
        patterns: Vec<ReplacePattern>,
    },
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

impl TemplateConfig {
    pub fn load_from_template_dir(template_dir: &Path) -> anyhow::Result<Self> {
        let path = template_dir.join("templator.json");
        if !path.exists() {
            anyhow::bail!(
                "Template config not found at {}. Place templator.json in the template directory.",
                path.display()
            );
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Cannot read config {}: {}", path.display(), e))?;
        serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid config JSON in {}: {}", path.display(), e))
    }
}

impl ProjectAnswers {
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read answers file {}: {}", path.display(), e))?;
        serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid answers JSON in {}: {}", path.display(), e))
    }

    pub fn validate_against(&self, config: &TemplateConfig) -> anyhow::Result<()> {
        for (var_name, var_def) in &config.variables {
            if !self.variables.contains_key(var_name) && var_def.default.is_none() {
                anyhow::bail!("Required variable missing in answers: {}", var_name);
            }
        }
        Ok(())
    }

    pub fn resolve(&self, config: &TemplateConfig) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (var_name, var_def) in &config.variables {
            let value = self
                .variables
                .get(var_name)
                .cloned()
                .or_else(|| var_def.default.clone())
                .unwrap_or_default();
            result.insert(var_name.clone(), value);
        }
        result
    }
}

impl SavedAnswers {
    pub fn write_to_output(&self, output_dir: &Path) -> anyhow::Result<()> {
        let path = output_dir.join(".templator-answers.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_minimal_template_config() {
        let json = r#"{"name": "test-template", "variables": {}}"#;
        let config: TemplateConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "test-template");
        assert!(config.description.is_none());
        assert!(config.post_processing.is_empty());
    }

    #[test]
    fn test_parse_template_config_with_variable_definitions() {
        let json = r#"{
            "name": "my-template",
            "variables": {
                "project_name": {},
                "author": {"default": "Anonymous"}
            }
        }"#;
        let config: TemplateConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.variables.len(), 2);
        assert!(config.variables["project_name"].default.is_none());
        assert_eq!(
            config.variables["author"].default.as_deref(),
            Some("Anonymous")
        );
    }

    #[test]
    fn test_parse_variable_definition_with_default() {
        let json = r#"{
            "name": "test",
            "variables": {
                "name": {"default": "foo"}
            }
        }"#;
        let config: TemplateConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.variables["name"].default.as_deref(), Some("foo"));
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
        let config: TemplateConfig = serde_json::from_str(json).unwrap();
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
    fn test_template_config_from_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_file = temp_dir.path().join("templator.json");
        write!(
            std::fs::File::create(&config_file).unwrap(),
            r#"{{"name": "x", "variables": {{}}, "post_processing": []}}"#
        )
        .unwrap();
        let config = TemplateConfig::load_from_template_dir(temp_dir.path()).unwrap();
        assert_eq!(config.name, "x");
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
        let config: TemplateConfig = serde_json::from_str(json).unwrap();
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
    fn test_project_answers_load_from_file() {
        let mut f = NamedTempFile::new().unwrap();
        write!(
            f,
            r#"{{"variables": {{"project_name": "my_cli", "author": "Darius"}}}}"#
        )
        .unwrap();
        let answers = ProjectAnswers::load_from_file(f.path()).unwrap();
        assert_eq!(answers.variables["project_name"], "my_cli");
        assert_eq!(answers.variables["author"], "Darius");
    }

    #[test]
    fn test_answers_validate_missing_required_variable() {
        let json = r#"{
            "name": "test",
            "variables": {
                "required": {"type": "string"},
                "optional": {"type": "string", "default": "def"}
            }
        }"#;
        let config: TemplateConfig = serde_json::from_str(json).unwrap();
        let answers_json = r#"{"variables": {"optional": "value"}}"#;
        let answers: ProjectAnswers = serde_json::from_str(answers_json).unwrap();
        assert!(answers.validate_against(&config).is_err());
    }

    #[test]
    fn test_answers_validate_success() {
        let json = r#"{
            "name": "test",
            "variables": {
                "required": {"type": "string"},
                "optional": {"type": "string", "default": "def"}
            }
        }"#;
        let config: TemplateConfig = serde_json::from_str(json).unwrap();
        let answers_json = r#"{"variables": {"required": "value", "optional": "custom"}}"#;
        let answers: ProjectAnswers = serde_json::from_str(answers_json).unwrap();
        assert!(answers.validate_against(&config).is_ok());
    }

    #[test]
    fn test_answers_resolve_merges_defaults() {
        let json = r#"{
            "name": "test",
            "variables": {
                "provided": {"type": "string"},
                "with_default": {"type": "string", "default": "default_value"}
            }
        }"#;
        let config: TemplateConfig = serde_json::from_str(json).unwrap();
        let answers_json = r#"{"variables": {"provided": "custom_value"}}"#;
        let answers: ProjectAnswers = serde_json::from_str(answers_json).unwrap();
        let resolved = answers.resolve(&config);
        assert_eq!(resolved["provided"], "custom_value");
        assert_eq!(resolved["with_default"], "default_value");
    }

    #[test]
    fn test_parse_null_name_errors() {
        let json = r#"{"name": null, "variables": {}}"#;
        assert!(serde_json::from_str::<TemplateConfig>(json).is_err());
    }

    #[test]
    fn test_parse_unknown_post_process_type_errors() {
        let json = r#"{"name":"t","variables":{},"post_processing":[{"type":"unknown_step"}]}"#;
        assert!(serde_json::from_str::<TemplateConfig>(json).is_err());
    }

    #[test]
    fn test_parse_unknown_pre_process_type_errors() {
        let json = r#"{"name":"t","variables":{},"pre_processing":[{"type":"unknown_step"}]}"#;
        assert!(serde_json::from_str::<TemplateConfig>(json).is_err());
    }
}
