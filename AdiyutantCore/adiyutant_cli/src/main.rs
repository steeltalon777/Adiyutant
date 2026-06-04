use adiyutant_store::Store;
use clap::Parser;

#[derive(Parser)]
#[command(name = "adiyutant")]
#[command(about = "Adiyutant — local-first organizer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Show current status
    Status,
    /// Health check
    Health,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status) => {
            println!("Adiyutant Core v0.1.0 — status: ok");
        }
        Some(Commands::Health) => {
            let store = adiyutant_store::NoopStore;
            match store.health_check() {
                Ok(()) => println!("health: ok"),
                Err(e) => eprintln!("health: {e}"),
            }
        }
        None => {
            println!("Adiyutant CLI v0.1.0. Use --help for commands.");
        }
    }
}
