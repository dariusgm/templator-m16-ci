use crate::config::PreProcessStep;

pub struct PreProcessor;

impl PreProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn effective_exclude(&self, steps: &[PreProcessStep]) -> Vec<String> {
        let mut patterns = Vec::new();
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
        let result = processor.effective_exclude(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_effective_exclude_single_step() {
        let processor = PreProcessor::new();
        let step = PreProcessStep::Exclude {
            patterns: vec!["*.log".to_string(), "temp/".to_string()],
        };
        let result = processor.effective_exclude(&[step]);
        assert_eq!(result, vec!["*.log".to_string(), "temp/".to_string()]);
    }

    #[test]
    fn test_effective_exclude_multiple_steps() {
        let processor = PreProcessor::new();
        let step1 = PreProcessStep::Exclude {
            patterns: vec!["*.log".to_string()],
        };
        let step2 = PreProcessStep::Exclude {
            patterns: vec!["temp/".to_string()],
        };
        let result = processor.effective_exclude(&[step1, step2]);
        assert_eq!(result, vec!["*.log".to_string(), "temp/".to_string()]);
    }

    #[test]
    fn test_effective_exclude_duplicates_allowed() {
        let processor = PreProcessor::new();
        let step1 = PreProcessStep::Exclude {
            patterns: vec!["*.log".to_string()],
        };
        let step2 = PreProcessStep::Exclude {
            patterns: vec!["*.log".to_string()],
        };
        let result = processor.effective_exclude(&[step1, step2]);
        assert_eq!(result, vec!["*.log".to_string(), "*.log".to_string()]);
    }
}
