use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use templator::{config::PostProcessStep, Config, PostProcessor, TemplateEngine};
use walkdir::WalkDir;

fn basic_template_dir() -> PathBuf {
    PathBuf::from("examples/basic-template/template")
}

fn basic_config_file() -> PathBuf {
    PathBuf::from("examples/basic-template/templator.json")
}

#[test]
fn test_basic_generation_files_exist() -> anyhow::Result<()> {
    let config = Config::from_file(&basic_config_file())?;
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(&basic_template_dir(), out.path(), &config)?;

    // Template files rendered without .template suffix
    assert!(
        out.path().join("pyproject.toml").exists(),
        "pyproject.toml missing"
    );
    assert!(
        out.path().join("src/my_project/main.py").exists(),
        "main.py missing"
    );

    // Non-template file copied as-is
    assert!(out.path().join(".gitignore").exists(), ".gitignore missing");

    Ok(())
}

#[test]
fn test_no_template_files_in_output() -> anyhow::Result<()> {
    let config = Config::from_file(&basic_config_file())?;
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(&basic_template_dir(), out.path(), &config)?;

    for entry in WalkDir::new(out.path()).into_iter().filter_map(|e| e.ok()) {
        let ext = entry.path().extension().unwrap_or_default();
        assert_ne!(
            ext,
            "template",
            "Unexpected .template file in output: {:?}",
            entry.path()
        );
    }

    Ok(())
}

#[test]
fn test_pyproject_toml_content() -> anyhow::Result<()> {
    let config = Config::from_file(&basic_config_file())?;
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(&basic_template_dir(), out.path(), &config)?;

    let content = fs::read_to_string(out.path().join("pyproject.toml"))?;
    assert!(
        content.contains(r#"name = "my_project""#),
        "project_name not rendered"
    );
    assert!(
        content.contains("py312"),
        "python_version_short not rendered"
    );
    assert!(content.contains("3.12"), "python_version not rendered");
    assert!(
        !content.contains("{{"),
        "Tera syntax left over in pyproject.toml"
    );

    Ok(())
}

#[test]
fn test_main_py_content() -> anyhow::Result<()> {
    let config = Config::from_file(&basic_config_file())?;
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(&basic_template_dir(), out.path(), &config)?;

    let content = fs::read_to_string(out.path().join("src/my_project/main.py"))?;
    assert!(
        content.contains("my_project"),
        "project_name not rendered in main.py"
    );
    assert!(
        content.contains("A Python project template"),
        "description not rendered"
    );
    assert!(!content.contains("{{"), "Tera syntax left over in main.py");

    Ok(())
}

#[test]
fn test_gitignore_copied_verbatim() -> anyhow::Result<()> {
    let config = Config::from_file(&basic_config_file())?;
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(&basic_template_dir(), out.path(), &config)?;

    let source = fs::read("examples/basic-template/template/.gitignore")?;
    let generated = fs::read(out.path().join(".gitignore"))?;
    assert_eq!(
        source, generated,
        ".gitignore should be a byte-identical copy"
    );

    Ok(())
}

#[test]
fn test_post_processing_replace() -> anyhow::Result<()> {
    let config = Config::from_file(&basic_config_file())?;
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(&basic_template_dir(), out.path(), &config)?;

    // Before post-processing, __AUTHOR__ placeholder is present
    let before = fs::read_to_string(out.path().join("pyproject.toml"))?;
    assert!(
        before.contains("__AUTHOR__"),
        "__AUTHOR__ should be present before post-processing"
    );

    PostProcessor::new(out.path().to_path_buf()).execute(&config.post_processing)?;

    let after = fs::read_to_string(out.path().join("pyproject.toml"))?;
    assert!(after.contains("Darius"), "Author not replaced");
    assert!(
        !after.contains("__AUTHOR__"),
        "__AUTHOR__ still present after post-processing"
    );

    Ok(())
}

#[test]
fn test_path_substitution_creates_correct_structure() -> anyhow::Result<()> {
    let config = Config::from_file(&basic_config_file())?;
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(&basic_template_dir(), out.path(), &config)?;

    // __project_name__ in path → my_project
    assert!(out.path().join("src/my_project").is_dir());
    assert!(out.path().join("src/my_project/main.py").exists());

    // The __project_name__ directory should NOT exist in output
    assert!(!out.path().join("src/__project_name__").exists());

    Ok(())
}

#[test]
fn test_tera_syntax_error_returns_error() {
    let template_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();

    fs::write(
        template_dir.path().join("broken.txt.template"),
        "{{ unclosed",
    )
    .unwrap();

    let config = Config {
        name: "test".into(),
        description: None,
        variables: HashMap::new(),
        exclude: vec![],
        post_processing: vec![],
    };

    let result = TemplateEngine::new().generate(template_dir.path(), output_dir.path(), &config);
    assert!(
        result.is_err(),
        "Invalid Tera syntax should propagate as error"
    );
}

#[test]
fn test_run_command_failure_aborts_post_processing() -> anyhow::Result<()> {
    let config = Config::from_file(&basic_config_file())?;
    let out = tempfile::tempdir()?;

    TemplateEngine::new().generate(&basic_template_dir(), out.path(), &config)?;

    let steps = vec![PostProcessStep::RunCommand {
        command: "exit 1".into(),
    }];
    let result = PostProcessor::new(out.path().to_path_buf()).execute(&steps);
    assert!(
        result.is_err(),
        "run_command failure should abort post-processing"
    );

    Ok(())
}

#[test]
fn test_multiple_path_substitutions_end_to_end() -> anyhow::Result<()> {
    let template_dir = tempfile::tempdir()?;
    let output_dir = tempfile::tempdir()?;

    // Two __var__ placeholders in the same path
    let src = template_dir
        .path()
        .join("src")
        .join("__org__")
        .join("__project__");
    fs::create_dir_all(&src)?;
    fs::write(src.join("main.py.template"), "# {{ org }}.{{ project }}")?;

    let config = Config {
        name: "test".into(),
        description: None,
        variables: HashMap::from([
            ("org".into(), "acme".into()),
            ("project".into(), "widget".into()),
        ]),
        exclude: vec![],
        post_processing: vec![],
    };

    TemplateEngine::new().generate(template_dir.path(), output_dir.path(), &config)?;

    let out_file = output_dir.path().join("src/acme/widget/main.py");
    assert!(out_file.exists(), "nested path substitution failed");
    assert_eq!(fs::read_to_string(out_file)?, "# acme.widget");

    Ok(())
}
