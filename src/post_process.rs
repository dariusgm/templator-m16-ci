use std::path::{Component, Path, PathBuf};
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

    fn preserve_sections(&self, _file: &str, _marker: &str) -> anyhow::Result<()> {
        anyhow::bail!("preserve_sections is not yet implemented");
    }

    fn replace_in_file(&self, file: &str, patterns: &[ReplacePattern]) -> anyhow::Result<()> {
        // Reject absolute paths and path traversal before any filesystem operation
        let file_path = Path::new(file);
        if file_path.is_absolute() || file_path.components().any(|c| c == Component::ParentDir) {
            anyhow::bail!(
                "replace path '{}' must be relative and non-traversing",
                file
            );
        }

        let path = self.working_dir.join(file);

        // Resolve symlinks to prevent escaping working_dir via symlink indirection
        let canonical_working_dir = self.working_dir.canonicalize()?;
        let canonical_path = path
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("Cannot read '{}': {}", file, e))?;
        if !canonical_path.starts_with(&canonical_working_dir) {
            anyhow::bail!("replace path '{}' resolves outside output directory", file);
        }

        let mut content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| anyhow::anyhow!("Cannot read '{}': {}", file, e))?;
        for pattern in patterns {
            content = content.replace(&pattern.from, &pattern.to);
        }
        std::fs::write(&canonical_path, content)?;
        log::info!("replace: applied {} pattern(s) to {}", patterns.len(), file);
        Ok(())
    }

    fn run_command(&self, command: &str) -> anyhow::Result<()> {
        log::info!("run_command: {}", command);
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to spawn command '{}': {}", command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}\n{}", command, stderr);
        }
        log::info!("run_command: done");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn processor(dir: &std::path::Path) -> PostProcessor {
        PostProcessor::new(dir.to_path_buf())
    }

    #[test]
    fn test_replace_single_pattern() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("file.txt"), "Hello __NAME__!").unwrap();

        processor(dir.path())
            .replace_in_file(
                "file.txt",
                &[ReplacePattern {
                    from: "__NAME__".into(),
                    to: "Alice".into(),
                }],
            )
            .unwrap();

        assert_eq!(
            fs::read_to_string(dir.path().join("file.txt")).unwrap(),
            "Hello Alice!"
        );
    }

    #[test]
    fn test_replace_multiple_patterns() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "__A__ + __B__").unwrap();

        processor(dir.path())
            .replace_in_file(
                "f.txt",
                &[
                    ReplacePattern {
                        from: "__A__".into(),
                        to: "foo".into(),
                    },
                    ReplacePattern {
                        from: "__B__".into(),
                        to: "bar".into(),
                    },
                ],
            )
            .unwrap();

        assert_eq!(
            fs::read_to_string(dir.path().join("f.txt")).unwrap(),
            "foo + bar"
        );
    }

    #[test]
    fn test_replace_all_occurrences() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "__X__ __X__ __X__").unwrap();

        processor(dir.path())
            .replace_in_file(
                "f.txt",
                &[ReplacePattern {
                    from: "__X__".into(),
                    to: "y".into(),
                }],
            )
            .unwrap();

        assert_eq!(
            fs::read_to_string(dir.path().join("f.txt")).unwrap(),
            "y y y"
        );
    }

    #[test]
    fn test_replace_missing_file_errors() {
        let dir = tempdir().unwrap();
        let result = processor(dir.path()).replace_in_file("nope.txt", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_dotdot_path_rejected() {
        let dir = tempdir().unwrap();
        let result = processor(dir.path()).replace_in_file("../secret.txt", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-traversing"));
    }

    #[test]
    fn test_replace_absolute_path_rejected() {
        let dir = tempdir().unwrap();
        let result = processor(dir.path()).replace_in_file("/etc/passwd", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-traversing"));
    }

    #[test]
    fn test_preserve_sections_not_implemented() {
        use crate::config::PostProcessStep;
        let dir = tempdir().unwrap();
        let steps = vec![PostProcessStep::PreserveSections {
            file: "f.txt".into(),
            marker: "---".into(),
        }];
        let result = processor(dir.path()).execute(&steps);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_command_success() {
        let dir = tempdir().unwrap();
        processor(dir.path()).run_command("true").unwrap();
    }

    #[test]
    fn test_run_command_creates_file() {
        let dir = tempdir().unwrap();
        processor(dir.path())
            .run_command("touch created.txt")
            .unwrap();
        assert!(dir.path().join("created.txt").exists());
    }

    #[test]
    fn test_run_command_failure_errors() {
        let dir = tempdir().unwrap();
        let result = processor(dir.path()).run_command("exit 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_multiple_steps() {
        use crate::config::PostProcessStep;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "__FOO__").unwrap();

        let steps = vec![
            PostProcessStep::Replace {
                file: "a.txt".into(),
                patterns: vec![ReplacePattern {
                    from: "__FOO__".into(),
                    to: "replaced".into(),
                }],
            },
            PostProcessStep::RunCommand {
                command: "touch done.txt".into(),
            },
        ];

        processor(dir.path()).execute(&steps).unwrap();

        assert_eq!(
            fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "replaced"
        );
        assert!(dir.path().join("done.txt").exists());
    }
}
