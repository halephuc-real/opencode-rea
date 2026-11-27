use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rea-launcher", version, about = "Run OpenCode with an abliterated model and REA")]
pub struct Cli {
    #[arg(long, default_value = "nanogpt")]
    pub provider: String,

    #[arg(long)]
    pub model: Option<String>,

    #[arg(long)]
    pub ask: bool,

    #[arg(long)]
    pub print_config: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Setup(SetupArgs),
    Logout,
    Version,
}

#[derive(Parser, Debug, Clone)]
pub struct SetupArgs {
    #[arg(long)]
    pub ghidra: Option<String>,
    #[arg(long, name = "java-home")]
    pub java_home: Option<String>,
    #[arg(long)]
    pub provider: Option<String>,
}
