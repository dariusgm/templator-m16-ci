use std::fs;
use std::path::PathBuf;
use templator::{PostProcessor, ProjectAnswers, SavedAnswers, TemplateConfig, TemplateEngine};

fn basic_template_package() -> PathBuf {
    PathBuf::from("examples/basic-template")
}

fn basic_answers_file() -> PathBuf {
    PathBuf::from("examples/basic-answers.json")
}

#[test]
fn test_basic_generation_files_exist() -> anyhow::Result<()> {
    let config = TemplateConfig::load_from_template_dir(&basic_template_package())?;
    let answers = ProjectAnswers::load_from_file(&basic_answers_file())?;
    let variables = answers.resolve(&config);
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(
        &basic_template_package().join("template"),
        out.path(),
        &config,
        &variables,
    )?;

    assert!(
        out.path().join("pyproject.toml").exists(),
        "pyproject.toml missing"
    );
    assert!(
        out.path().join("src/my_project/main.py").exists(),
        "main.py missing"
    );
    assert!(out.path().join(".gitignore").exists(), ".gitignore missing");

    Ok(())
}

#[test]
fn test_post_processing_and_saved_answers() -> anyhow::Result<()> {
    let config = TemplateConfig::load_from_template_dir(&basic_template_package())?;
    let answers = ProjectAnswers::load_from_file(&basic_answers_file())?;
    let variables = answers.resolve(&config);
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(
        &basic_template_package().join("template"),
        out.path(),
        &config,
        &variables,
    )?;

    if !config.post_processing.is_empty() {
        PostProcessor::new(out.path().to_path_buf()).execute(&config.post_processing)?;
    }

    let saved = SavedAnswers {
        _template: "examples/basic-template".to_string(),
        _generated_at: chrono::Utc::now().to_rfc3339(),
        variables,
    };
    saved.write_to_output(out.path())?;

    assert!(
        out.path().join(".templator-answers.json").exists(),
        ".templator-answers.json not found"
    );

    let content = fs::read_to_string(out.path().join("pyproject.toml"))?;
    assert!(
        content.contains("my_project"),
        "Expected 'my_project' in pyproject.toml"
    );

    Ok(())
}
