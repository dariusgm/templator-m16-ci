use chrono::Utc;
use clap::Parser;
use std::path::PathBuf;
use templator::{PostProcessor, ProjectAnswers, SavedAnswers, TemplateConfig, TemplateEngine};

#[derive(Parser)]
#[command(name = "templator")]
#[command(
    about = "Generate projects from templates. Reads templator.json from the template directory and applies variable substitutions."
)]
#[command(version)]
struct Args {
    /// Template package directory (must contain templator.json)
    #[arg(long)]
    template: PathBuf,

    /// Project answers JSON file with variable values
    #[arg(long)]
    answers: PathBuf,

    /// Output directory (created if it does not exist)
    #[arg(long)]
    output: PathBuf,

    /// Enable verbose (debug) logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let log_level = if args.verbose { "debug" } else { "info" };
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", log_level);
    }
    env_logger::init();

    if !args.template.exists() {
        anyhow::bail!(
            "Template package directory not found: {}",
            args.template.display()
        );
    }
    if !args.answers.exists() {
        anyhow::bail!("Answers file not found: {}", args.answers.display());
    }

    let config = TemplateConfig::load_from_template_dir(&args.template)?;
    let answers = ProjectAnswers::load_from_file(&args.answers)?;

    println!("Generating: {}", config.name);

    answers.validate_against(&config)?;
    let variables = answers.resolve(&config);

    let template_content_dir = args.template.join("template");
    TemplateEngine::new().generate(&template_content_dir, &args.output, &config, &variables)?;
    println!("Files generated.");

    if !config.post_processing.is_empty() {
        PostProcessor::new(args.output.clone()).execute(&config.post_processing)?;
        println!("Post-processing done.");
    }

    let saved_answers = SavedAnswers {
        _template: args.template.display().to_string(),
        _generated_at: Utc::now().to_rfc3339(),
        variables,
    };
    saved_answers.write_to_output(&args.output)?;
    println!("Project ready: {}", args.output.display());
    Ok(())
}
