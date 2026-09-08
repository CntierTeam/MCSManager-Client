use anyhow::Result;
use clap::Parser;
use mcsm_cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    // No args (only binary name) → TUI. Any args → CLI (including --help).
    if std::env::args_os().len() <= 1 {
        return mcsm_tui::run().await;
    }

    let cli = Cli::parse();
    if cli.command.is_none() {
        // Flags only (e.g. --url) without subcommand → still prefer showing help via clap
        // by re-parsing with required subcommand behavior: launch TUI if literally no useful cmd.
        // If user passed global flags only, run TUI with those env-applied via config overrides.
        return mcsm_tui::run().await;
    }
    mcsm_cli::run(cli).await
}
