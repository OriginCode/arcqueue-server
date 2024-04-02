use clap::Parser;

/// Arcade Queue Server
#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// PostgreSQL Server URL
    #[arg(short, long)]
    pub pg_url: String,
}
