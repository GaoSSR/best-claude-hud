use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "best-claude-hud")]
#[command(version, about = "Minimal Claude Code statusline HUD")]
pub struct Cli {
    /// Enter TUI configuration mode
    #[arg(short = 'c', long = "config")]
    pub config: bool,

    /// Set theme
    #[arg(short = 't', long = "theme")]
    pub theme: Option<String>,

    /// Configure Claude Code to use best-claude-hud as the statusline command
    #[arg(long = "setup")]
    pub setup: bool,

    /// Patch Claude Code cli.js to disable context warnings
    #[arg(long = "patch")]
    pub patch: Option<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
