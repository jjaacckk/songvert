mod cli;

use crate::cli::Cli;
use clap::Parser;
use songvert::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let cli = Cli::parse();
    cli.run().await?;

    Ok(())
}
