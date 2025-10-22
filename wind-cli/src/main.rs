use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing_subscriber;

mod commands;

#[derive(Parser)]
#[command(name = "wind")]
#[command(version, about = "A modern version control system built on Git", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize a new Wind repository")]
    Init {
        #[arg(help = "Path to initialize (default: current directory)")]
        path: Option<String>,
    },

    #[command(about = "Show working tree status")]
    #[command(alias = "st")]
    Status,

    #[command(about = "Add files to staging area")]
    #[command(alias = "stage")]
    Add {
        #[arg(help = "Files to add")]
        files: Vec<String>,
        #[arg(short, long, help = "Add all changes")]
        all: bool,
    },

    #[command(about = "Record changes to the repository")]
    Commit {
        #[arg(short, long, help = "Commit message")]
        message: Option<String>,
        #[arg(short, long, help = "Use AI to suggest commit message")]
        ai: bool,
    },

    #[command(about = "Show commit history")]
    Log {
        #[arg(short, long, help = "Number of commits to show")]
        n: Option<usize>,
        #[arg(long, help = "Show graph")]
        graph: bool,
    },

    #[command(about = "List, create, or delete branches")]
    Branch {
        #[arg(help = "Branch name to create")]
        name: Option<String>,
        #[arg(short, long, help = "Delete branch")]
        delete: bool,
        #[arg(short, long, help = "List all branches")]
        list: bool,
    },

    #[command(about = "Switch branches or restore working tree files")]
    Checkout {
        #[arg(help = "Branch or commit to checkout")]
        target: String,
    },

    #[command(about = "Manage stacks of dependent branches")]
    Stack {
        #[command(subcommand)]
        action: StackAction,
    },

    #[command(about = "Reapply commits on top of another base")]
    Rebase {
        #[arg(help = "Branch to rebase onto")]
        onto: String,
    },

    #[command(about = "Resolve merge conflicts interactively")]
    Resolve {
        #[arg(help = "File to resolve (if omitted, lists all conflicts)")]
        file: Option<String>,
    },

    #[command(about = "Create and manage pull requests")]
    Pr {
        #[command(subcommand)]
        action: PrAction,
    },

    #[command(about = "Launch interactive terminal UI")]
    Tui,

    #[command(about = "Configure AI features")]
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },

    #[command(about = "Get and set repository or global options")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(about = "Manage worktrees")]
    Worktree {
        #[command(subcommand)]
        action: WorktreeAction,
    },

    #[command(about = "Manage submodules")]
    Submodule {
        #[command(subcommand)]
        action: SubmoduleAction,
    },

    #[command(about = "Sync Wind with Git repository")]
    Sync {
        #[arg(short, long, help = "Quiet mode")]
        quiet: bool,
        #[arg(long, help = "Install Git hooks")]
        install: bool,
    },

    #[command(about = "Import existing Git repository to Wind")]
    ImportGit {
        #[arg(help = "Path to Git repository (default: current directory)")]
        path: Option<String>,
    },

    #[command(about = "Export Wind repository to Git")]
    ExportGit {
        #[arg(help = "Path for exported Git repository")]
        path: String,
    },
    
    #[command(about = "Push changes to remote (exports to Git then pushes)")]
    Push {
        #[arg(help = "Remote name", default_value = "origin")]
        remote: String,
        #[arg(help = "Branch name (defaults to current branch)")]
        branch: Option<String>,
    },
}

#[derive(Subcommand)]
enum StackAction {
    #[command(about = "List all stacks")]
    List,
    #[command(about = "Create a new stack")]
    Create {
        #[arg(help = "Stack name")]
        name: String,
    },
    #[command(about = "Rebase entire stack")]
    Rebase,
    #[command(about = "Land/merge stack to main")]
    Land,
}

#[derive(Subcommand)]
enum PrAction {
    #[command(about = "Create a new pull request")]
    Create {
        #[arg(short, long, help = "PR title")]
        title: Option<String>,
        #[arg(short, long, help = "PR description")]
        body: Option<String>,
    },
    #[command(about = "Update existing pull request")]
    Update {
        #[arg(help = "PR number")]
        number: u32,
    },
    #[command(about = "List pull requests")]
    List,
}

#[derive(Subcommand)]
enum AiAction {
    #[command(about = "Enable AI features")]
    Enable,
    #[command(about = "Disable AI features")]
    Disable,
    #[command(about = "Configure AI provider")]
    Configure {
        #[arg(long, help = "API key")]
        api_key: Option<String>,
        #[arg(long, help = "Provider (openai, anthropic, local)")]
        provider: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    #[command(about = "Get a configuration value")]
    Get {
        #[arg(help = "Configuration key")]
        key: String,
    },
    #[command(about = "Set a configuration value")]
    Set {
        #[arg(help = "Configuration key")]
        key: String,
        #[arg(help = "Configuration value")]
        value: String,
    },
    #[command(about = "List all configuration")]
    List,
}

#[derive(Subcommand)]
enum WorktreeAction {
    #[command(about = "List all worktrees")]
    List,
    #[command(about = "Add a new worktree")]
    Add {
        #[arg(help = "Path for the new worktree")]
        path: String,
        #[arg(help = "Branch to checkout in the worktree")]
        branch: Option<String>,
    },
    #[command(about = "Remove a worktree")]
    Remove {
        #[arg(help = "Path of the worktree to remove")]
        path: String,
    },
}

#[derive(Subcommand)]
enum SubmoduleAction {
    #[command(about = "List all submodules")]
    List,
    #[command(about = "Show submodule status")]
    Status,
    #[command(about = "Initialize submodule(s)")]
    Init {
        #[arg(help = "Specific submodule name (optional)")]
        name: Option<String>,
    },
    #[command(about = "Update submodule(s)")]
    Update {
        #[arg(help = "Specific submodule name (optional)")]
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        eprintln!("\n{}", "Interrupted by user".yellow());
        std::process::exit(130);
    })?;

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { path } => commands::init::execute(path).await,
        Commands::Status => commands::status::execute().await,
        Commands::Add { files, all } => commands::add::execute(files, all).await,
        Commands::Commit { message, ai } => commands::commit::execute(message, ai).await,
        Commands::Log { n, graph } => commands::log::execute(n, graph).await,
        Commands::Branch { name, delete, list } => {
            commands::branch::execute(name, delete, list).await
        }
        Commands::Checkout { target } => commands::checkout::execute(target).await,
        Commands::Stack { action } => commands::stack::execute(action).await,
        Commands::Rebase { onto } => commands::rebase::execute(onto).await,
        Commands::Resolve { file } => commands::resolve::execute(file).await,
        Commands::Pr { action } => commands::pr::execute(action).await,
        Commands::Tui => commands::tui::execute().await,
        Commands::Ai { action } => commands::ai::execute(action).await,
        Commands::Config { action } => commands::config::execute(action).await,
        Commands::Push { remote, branch } => commands::push::execute(remote, branch).await,
        Commands::Worktree { action } => commands::worktree::execute(action).await,
        Commands::Submodule { action } => commands::submodule::execute(action).await,
        Commands::Sync { quiet, install } => commands::sync::handle_sync(quiet, install),
        Commands::ImportGit { path } => {
            commands::import::execute(path.unwrap_or_else(|| ".".to_string())).await
        }
        Commands::ExportGit { path } => commands::export::execute(path).await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}
