use clap::Parser;

/// Arcade Queue Server
#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// PostgreSQL Server URL
    #[arg(short = 'u', long)]
    pub pg_url: String,
    #[arg(short, long)]
    pub port: Option<u16>,
    #[arg(short = 'l', long)]
    pub host: Option<String>,
}
