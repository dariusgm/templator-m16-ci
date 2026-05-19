use std::fs;
use std::path::PathBuf;
use templator::{Config, PostProcessor, TemplateEngine};
use walkdir::WalkDir;

fn basic_template_dir() -> PathBuf {
    PathBuf::from("examples/basic-template/template")
}

fn basic_config_file() -> PathBuf {
    PathBuf::from("examples/basic-template/config.json")
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
