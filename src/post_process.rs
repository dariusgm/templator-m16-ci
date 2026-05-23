use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use crate::config::{PostProcessStep, ReplacePattern};

pub struct PostProcessor {
    working_dir: PathBuf,
    preserved_sections: HashMap<String, Vec<String>>,
}

impl PostProcessor {
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            preserved_sections: HashMap::new(),
        }
    }

    pub fn snapshot_preserved_sections(&mut self, steps: &[PostProcessStep]) {
        for step in steps {
            if let PostProcessStep::PreserveSections {
                file,
                begin_marker,
                end_marker,
            } = step
            {
                let file_path = Path::new(file);
                if file_path.is_absolute()
                    || file_path.components().any(|c| c == Component::ParentDir)
                {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(self.working_dir.join(file)) {
                    let sections = extract_user_sections(&content, begin_marker, end_marker);
                    if !sections.is_empty() {
                        log::debug!(
                            "preserve_sections: snapshotted {} section(s) from '{}'",
                            sections.len(),
                            file
                        );
                        self.preserved_sections.insert(file.clone(), sections);
                    }
                }
            }
        }
    }

    pub fn execute(&self, steps: &[PostProcessStep]) -> anyhow::Result<()> {
        for step in steps {
            match step {
                PostProcessStep::PreserveSections {
                    file,
                    begin_marker,
                    end_marker,
                } => {
                    self.preserve_sections(file, begin_marker, end_marker)?;
                }
                PostProcessStep::Replace { file, patterns } => {
                    self.replace_in_file(file, patterns)?;
                }
            }
        }
        Ok(())
    }

    fn preserve_sections(
        &self,
        file: &str,
        begin_marker: &str,
        end_marker: &str,
    ) -> anyhow::Result<()> {
        let file_path = Path::new(file);
        if file_path.is_absolute() || file_path.components().any(|c| c == Component::ParentDir) {
            anyhow::bail!(
                "preserve_sections path '{}' must be relative and non-traversing",
                file
            );
        }
        let path = self.working_dir.join(file);
        let canonical_working_dir = self.working_dir.canonicalize()?;
        let canonical_path = path
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("Cannot read '{}': {}", file, e))?;
        if !canonical_path.starts_with(&canonical_working_dir) {
            anyhow::bail!(
                "preserve_sections path '{}' resolves outside output directory",
                file
            );
        }

        let generated = std::fs::read_to_string(&canonical_path)
            .map_err(|e| anyhow::anyhow!("Cannot read '{}': {}", file, e))?;

        let preserved = self
            .preserved_sections
            .get(file)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let merged = merge_user_sections(&generated, preserved, begin_marker, end_marker);

        if merged != generated {
            std::fs::write(&canonical_path, &merged)?;
            log::info!(
                "preserve_sections: merged {} section(s) into '{}'",
                preserved.len(),
                file
            );
        } else {
            log::debug!("preserve_sections: no change to '{}'", file);
        }
        Ok(())
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
}

fn extract_user_sections(content: &str, begin: &str, end: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut search_from = 0;
    while let Some(begin_pos) = content[search_from..].find(begin) {
        let begin_abs = search_from + begin_pos;
        let after_begin = begin_abs + begin.len();
        if let Some(end_pos) = content[after_begin..].find(end) {
            let end_abs = after_begin + end_pos;
            sections.push(content[after_begin..end_abs].to_string());
            search_from = end_abs + end.len();
        } else {
            break;
        }
    }
    sections
}

fn merge_user_sections(generated: &str, preserved: &[String], begin: &str, end: &str) -> String {
    let mut result = String::new();
    let mut search_from = 0;
    let mut section_index = 0;

    while let Some(begin_pos) = generated[search_from..].find(begin) {
        let begin_abs = search_from + begin_pos;
        let after_begin = begin_abs + begin.len();
        result.push_str(&generated[search_from..after_begin]);

        if let Some(end_pos) = generated[after_begin..].find(end) {
            let end_abs = after_begin + end_pos;
            if let Some(preserved_content) = preserved.get(section_index) {
                result.push_str(preserved_content);
            }
            search_from = end_abs;
            section_index += 1;
        } else {
            result.push_str(&generated[after_begin..]);
            return result;
        }
    }
    result.push_str(&generated[search_from..]);
    result
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
    fn test_extract_user_sections_single() {
        let content = "template\n# BEGIN\nuser stuff\n# END\nmore template";
        let sections = extract_user_sections(content, "# BEGIN\n", "# END\n");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0], "user stuff\n");
    }

    #[test]
    fn test_extract_user_sections_multiple() {
        let content = "t1\n# B\nu1\n# E\nt2\n# B\nu2\n# E\nt3";
        let sections = extract_user_sections(content, "# B\n", "# E\n");
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0], "u1\n");
        assert_eq!(sections[1], "u2\n");
    }

    #[test]
    fn test_extract_user_sections_no_markers() {
        let content = "just template content";
        let sections = extract_user_sections(content, "# BEGIN", "# END");
        assert!(sections.is_empty());
    }

    #[test]
    fn test_extract_user_sections_unmatched_begin() {
        let content = "template\n# BEGIN\nuser\nno end marker";
        let sections = extract_user_sections(content, "# BEGIN\n", "# END\n");
        assert!(sections.is_empty());
    }

    #[test]
    fn test_merge_preserves_user_sections() {
        let generated = "new template\n# B\n# E\nmore template";
        let preserved = vec!["user content\n".to_string()];
        let result = merge_user_sections(generated, &preserved, "# B\n", "# E\n");
        assert_eq!(
            result,
            "new template\n# B\nuser content\n# E\nmore template"
        );
    }

    #[test]
    fn test_merge_multiple_sections() {
        let generated = "t1\n# B\n# E\nt2\n# B\n# E\nt3";
        let preserved = vec!["u1\n".to_string(), "u2\n".to_string()];
        let result = merge_user_sections(generated, &preserved, "# B\n", "# E\n");
        assert_eq!(result, "t1\n# B\nu1\n# E\nt2\n# B\nu2\n# E\nt3");
    }

    #[test]
    fn test_merge_no_preserved_sections_first_generation() {
        let generated = "new template\n# B\ndefault\n# E\nmore";
        let preserved = &[];
        let result = merge_user_sections(generated, preserved, "# B\n", "# E\n");
        assert_eq!(result, "new template\n# B\n# E\nmore");
    }

    #[test]
    fn test_merge_extra_old_sections_discarded() {
        let generated = "t1\n# B\n# E\nt2";
        let preserved = vec!["u1\n".to_string(), "u2\n".to_string()];
        let result = merge_user_sections(generated, &preserved, "# B\n", "# E\n");
        assert_eq!(result, "t1\n# B\nu1\n# E\nt2");
    }

    #[test]
    fn test_preserve_sections_first_run_noop() {
        use crate::config::PostProcessStep;
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "template\n# BEGIN\n# END\nmore").unwrap();
        let proc = processor(dir.path());
        proc.execute(&[PostProcessStep::PreserveSections {
            file: "f.txt".into(),
            begin_marker: "# BEGIN\n".into(),
            end_marker: "# END\n".into(),
        }])
        .unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("f.txt")).unwrap(),
            "template\n# BEGIN\n# END\nmore"
        );
    }

    #[test]
    fn test_preserve_sections_restores_on_regen() {
        use crate::config::PostProcessStep;
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "old\n# BEGIN\nuser\n# END\nold2").unwrap();

        let mut proc = processor(dir.path());
        proc.snapshot_preserved_sections(&[PostProcessStep::PreserveSections {
            file: "f.txt".into(),
            begin_marker: "# BEGIN\n".into(),
            end_marker: "# END\n".into(),
        }]);

        fs::write(dir.path().join("f.txt"), "new\n# BEGIN\n# END\nnew2").unwrap();

        proc.execute(&[PostProcessStep::PreserveSections {
            file: "f.txt".into(),
            begin_marker: "# BEGIN\n".into(),
            end_marker: "# END\n".into(),
        }])
        .unwrap();

        assert_eq!(
            fs::read_to_string(dir.path().join("f.txt")).unwrap(),
            "new\n# BEGIN\nuser\n# END\nnew2"
        );
    }

    #[test]
    fn test_preserve_sections_dotdot_rejected() {
        use crate::config::PostProcessStep;
        let dir = tempdir().unwrap();
        let mut proc = processor(dir.path());
        proc.preserved_sections
            .insert("../escape.txt".into(), vec!["\nuser\n".into()]);

        let result = proc.execute(&[PostProcessStep::PreserveSections {
            file: "../escape.txt".into(),
            begin_marker: "# B".into(),
            end_marker: "# E".into(),
        }]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-traversing"));
    }

    #[test]
    fn test_preserve_sections_absolute_rejected() {
        use crate::config::PostProcessStep;
        let dir = tempdir().unwrap();
        let mut proc = processor(dir.path());
        proc.preserved_sections
            .insert("/etc/passwd".into(), vec!["\nuser\n".into()]);

        let result = proc.execute(&[PostProcessStep::PreserveSections {
            file: "/etc/passwd".into(),
            begin_marker: "# B".into(),
            end_marker: "# E".into(),
        }]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-traversing"));
    }

    #[test]
    fn test_execute_multiple_steps() {
        use crate::config::PostProcessStep;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "__FOO__").unwrap();

        let steps = vec![PostProcessStep::Replace {
            file: "a.txt".into(),
            patterns: vec![ReplacePattern {
                from: "__FOO__".into(),
                to: "replaced".into(),
            }],
        }];

        processor(dir.path()).execute(&steps).unwrap();

        assert_eq!(
            fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "replaced"
        );
    }

    #[test]
    fn test_replace_empty_patterns_is_noop() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "unchanged content").unwrap();

        processor(dir.path()).replace_in_file("f.txt", &[]).unwrap();

        assert_eq!(
            fs::read_to_string(dir.path().join("f.txt")).unwrap(),
            "unchanged content"
        );
    }

    #[test]
    fn test_replace_patterns_applied_sequentially() {
        // Second pattern acts on the output of the first, not on the original text.
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "foo").unwrap();

        processor(dir.path())
            .replace_in_file(
                "f.txt",
                &[
                    ReplacePattern {
                        from: "foo".into(),
                        to: "bar".into(),
                    },
                    ReplacePattern {
                        from: "bar".into(),
                        to: "baz".into(),
                    },
                ],
            )
            .unwrap();

        assert_eq!(fs::read_to_string(dir.path().join("f.txt")).unwrap(), "baz");
    }

    #[test]
    fn test_replace_in_nested_subdirectory() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("sub/dir")).unwrap();
        fs::write(dir.path().join("sub/dir/file.txt"), "__PLACEHOLDER__").unwrap();

        processor(dir.path())
            .replace_in_file(
                "sub/dir/file.txt",
                &[ReplacePattern {
                    from: "__PLACEHOLDER__".into(),
                    to: "value".into(),
                }],
            )
            .unwrap();

        assert_eq!(
            fs::read_to_string(dir.path().join("sub/dir/file.txt")).unwrap(),
            "value"
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_replace_rejects_symlink_escaping_working_dir() {
        use std::os::unix::fs::symlink;

        let working_dir = tempdir().unwrap();
        let external_dir = tempdir().unwrap();

        let external_file = external_dir.path().join("secret.txt");
        fs::write(&external_file, "secret content").unwrap();

        // Symlink inside working_dir pointing to a file outside it
        symlink(&external_file, working_dir.path().join("link.txt")).unwrap();

        let result = processor(working_dir.path()).replace_in_file(
            "link.txt",
            &[ReplacePattern {
                from: "secret".into(),
                to: "replaced".into(),
            }],
        );

        assert!(result.is_err(), "Symlink escape should be rejected");
        // External file must not have been modified
        assert_eq!(
            fs::read_to_string(&external_file).unwrap(),
            "secret content"
        );
    }
}
