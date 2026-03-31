mod config;
mod daemon;
mod engine;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "thok", about = "Mechanical keyboard sound simulator")]
struct Cli {
    /// Set the active sound pack profile
    #[arg(long)]
    profile: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the sound engine in the background
    Start,
    /// Stop the sound engine
    Stop,
    /// Manage sound pack profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Internal: run the sound engine (used by `start`)
    #[command(hide = true)]
    Run,
}

#[derive(Subcommand)]
enum ProfileAction {
    /// List available sound packs
    List,
}

fn main() {
    let cli = Cli::parse();

    if let Some(profile) = &cli.profile {
        config::set_profile(profile);
        println!("Profile set to: {profile}");
    }

    match cli.command {
        Some(Commands::Start) => daemon::start(),
        Some(Commands::Stop) => daemon::stop(),
        Some(Commands::Profile { action }) => match action {
            ProfileAction::List => config::list_profiles(),
        },
        Some(Commands::Run) => engine::run(),
        None => {
            if cli.profile.is_none() {
                // No flag and no subcommand — show help
                use clap::CommandFactory;
                Cli::command().print_help().unwrap();
                println!();
            }
        }
    }
}
