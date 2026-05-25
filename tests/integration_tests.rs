use std::fs;
use std::path::PathBuf;
use templator::{PostProcessor, ProjectAnswers, SavedAnswers, TemplateConfig, TemplateEngine};

fn basic_template_package() -> PathBuf {
    PathBuf::from("examples/complete-example")
}

fn basic_answers_file() -> PathBuf {
    PathBuf::from("examples/complete-example-answers.json")
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

#[test]
fn test_preserve_sections_pyproject_regeneration() -> anyhow::Result<()> {
    let template_pkg = PathBuf::from("examples/complete-example");
    let answers_file = PathBuf::from("examples/complete-example-answers.json");

    let config = TemplateConfig::load_from_template_dir(&template_pkg)?;
    let answers = ProjectAnswers::load_from_file(&answers_file)?;
    let variables = answers.resolve(&config);
    let out = tempfile::tempdir()?;

    // First generation
    TemplateEngine::new().generate(
        &template_pkg.join("template"),
        out.path(),
        &config,
        &variables,
    )?;

    let generated = fs::read_to_string(out.path().join("pyproject.toml"))?;
    assert!(
        generated.contains("my_project"),
        "Expected 'my_project' in generated pyproject.toml"
    );
    assert!(
        generated.contains("# [preserve:begin]"),
        "Expected begin marker in generated file"
    );
    assert!(
        generated.contains("# [preserve:end]"),
        "Expected end marker in generated file"
    );

    // Simulate user adding dependencies between markers
    let with_user_deps = generated.replace(
        "# [preserve:begin]\n# [preserve:end]",
        "# [preserve:begin]\nrequests = \">=2.31\"\npydantic = \">=2.0\"\n# [preserve:end]",
    );
    fs::write(out.path().join("pyproject.toml"), with_user_deps)?;

    // Simulate template update: change project name in answers
    let updated_answers_json =
        r#"{"variables":{"project_name":"updated_app","python_version":"3.12","author":"Alice"}}"#;
    let updated_answers: ProjectAnswers = serde_json::from_str(updated_answers_json)?;
    let updated_variables = updated_answers.resolve(&config);

    // Regenerate with snapshot → generate → post-process
    let mut post_processor = PostProcessor::new(out.path().to_path_buf());
    post_processor.snapshot_preserved_sections(&config.post_processing);

    TemplateEngine::new().generate(
        &template_pkg.join("template"),
        out.path(),
        &config,
        &updated_variables,
    )?;

    post_processor.execute(&config.post_processing)?;

    let final_content = fs::read_to_string(out.path().join("pyproject.toml"))?;

    // Verify template content was updated
    assert!(
        final_content.contains("updated_app"),
        "Expected 'updated_app' in regenerated pyproject.toml"
    );

    // Verify user content was preserved
    assert!(
        final_content.contains("requests = \">=2.31\""),
        "Expected preserved 'requests' dependency"
    );
    assert!(
        final_content.contains("pydantic = \">=2.0\""),
        "Expected preserved 'pydantic' dependency"
    );

    // Verify markers are still present
    assert!(
        final_content.contains("# [preserve:begin]"),
        "Expected begin marker in regenerated file"
    );
    assert!(
        final_content.contains("# [preserve:end]"),
        "Expected end marker in regenerated file"
    );

    // Verify structure: user deps are between markers
    let begin_pos = final_content.find("# [preserve:begin]").unwrap();
    let end_pos = final_content.find("# [preserve:end]").unwrap();
    let between_markers = &final_content[begin_pos..end_pos];

    assert!(
        between_markers.contains("requests"),
        "requests should be between markers"
    );
    assert!(
        between_markers.contains("pydantic"),
        "pydantic should be between markers"
    );

    Ok(())
}

#[cfg(unix)]
#[test]
fn test_generate_preserves_executable_bit_on_template_file() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let template_dir = tempfile::tempdir()?;
    let out = tempfile::tempdir()?;

    // Executable .template file — goes through fs::write path
    let src_template = template_dir.path().join("install.sh.template");
    fs::write(&src_template, "#!/bin/sh\necho {{ name }}")?;
    fs::set_permissions(&src_template, fs::Permissions::from_mode(0o755))?;

    // Executable plain file — verifies existing fs::copy path is correct
    let src_plain = template_dir.path().join("helper.sh");
    fs::write(&src_plain, "#!/bin/sh\necho helper")?;
    fs::set_permissions(&src_plain, fs::Permissions::from_mode(0o755))?;

    let config = TemplateConfig {
        name: "perm-test".into(),
        description: None,
        variables: std::collections::HashMap::new(),
        exclude: vec![],
        pre_processing: vec![],
        post_processing: vec![],
    };
    let variables = std::collections::HashMap::from([("name".to_string(), "world".to_string())]);

    TemplateEngine::new().generate(template_dir.path(), out.path(), &config, &variables)?;

    let rendered = out.path().join("install.sh");
    assert!(rendered.exists());
    assert_eq!(
        fs::metadata(&rendered)?.permissions().mode() & 0o111,
        0o111,
        "rendered template file must preserve executable bit"
    );

    let copied = out.path().join("helper.sh");
    assert!(copied.exists());
    assert_eq!(
        fs::metadata(&copied)?.permissions().mode() & 0o111,
        0o111,
        "verbatim-copied file must preserve executable bit"
    );

    Ok(())
}
