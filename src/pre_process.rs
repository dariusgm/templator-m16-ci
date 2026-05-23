use crate::config::PreProcessStep;

pub struct PreProcessor;

impl PreProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn effective_exclude(
        &self,
        base_exclude: &[String],
        steps: &[PreProcessStep],
    ) -> Vec<String> {
        let mut patterns = base_exclude.to_vec();
        for step in steps {
            match step {
                PreProcessStep::Exclude { patterns: extra } => {
                    patterns.extend(extra.iter().cloned());
                }
            }
        }
        patterns
    }
}

impl Default for PreProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_exclude_empty_inputs() {
        let processor = PreProcessor::new();
        let result = processor.effective_exclude(&[], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_effective_exclude_base_only() {
        let processor = PreProcessor::new();
        let base = vec!["*.tmp".to_string(), "build/".to_string()];
        let result = processor.effective_exclude(&base, &[]);
        assert_eq!(result, base);
    }

    #[test]
    fn test_effective_exclude_steps_only() {
        let processor = PreProcessor::new();
        let step = PreProcessStep::Exclude {
            patterns: vec!["*.log".to_string(), "temp/".to_string()],
        };
        let result = processor.effective_exclude(&[], &[step]);
        assert_eq!(result, vec!["*.log".to_string(), "temp/".to_string()]);
    }

    #[test]
    fn test_effective_exclude_merge_additive() {
        let processor = PreProcessor::new();
        let base = vec!["*.tmp".to_string()];
        let step = PreProcessStep::Exclude {
            patterns: vec!["build/".to_string()],
        };
        let result = processor.effective_exclude(&base, &[step]);
        assert_eq!(result, vec!["*.tmp".to_string(), "build/".to_string()]);
    }

    #[test]
    fn test_effective_exclude_multiple_steps_merged() {
        let processor = PreProcessor::new();
        let base = vec!["*.draft".to_string()];
        let step1 = PreProcessStep::Exclude {
            patterns: vec!["*.log".to_string()],
        };
        let step2 = PreProcessStep::Exclude {
            patterns: vec!["temp/".to_string()],
        };
        let result = processor.effective_exclude(&base, &[step1, step2]);
        assert_eq!(
            result,
            vec![
                "*.draft".to_string(),
                "*.log".to_string(),
                "temp/".to_string()
            ]
        );
    }

    #[test]
    fn test_effective_exclude_duplicates_allowed() {
        let processor = PreProcessor::new();
        let base = vec!["*.log".to_string()];
        let step = PreProcessStep::Exclude {
            patterns: vec!["*.log".to_string()],
        };
        let result = processor.effective_exclude(&base, &[step]);
        assert_eq!(result, vec!["*.log".to_string(), "*.log".to_string()]);
    }
}
