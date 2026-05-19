use clap::Parser;
use std::path::PathBuf;
use templator::{Config, PostProcessor, TemplateEngine};

#[derive(Parser)]
#[command(name = "templator")]
#[command(about = "Generate projects from templates. Only files with a .template suffix have their content rendered; all others are copied verbatim.")]
#[command(version)]
struct Args {
    /// Template directory containing source files
    #[arg(short, long)]
    template: PathBuf,

    /// JSON config file with variables and post-processing steps
    #[arg(short, long)]
    config: PathBuf,

    /// Output directory (created if it does not exist)
    #[arg(short, long)]
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
        anyhow::bail!("Template directory not found: {}", args.template.display());
    }
    if !args.config.exists() {
        anyhow::bail!("Config file not found: {}", args.config.display());
    }

    let config = Config::from_file(&args.config)?;
    println!("Generating: {}", config.name);

    TemplateEngine::new().generate(&args.template, &args.output, &config)?;
    println!("Files generated.");

    if !config.post_processing.is_empty() {
        PostProcessor::new(args.output.clone()).execute(&config.post_processing)?;
        println!("Post-processing done.");
    }

    println!("Project ready: {}", args.output.display());
    Ok(())
}
